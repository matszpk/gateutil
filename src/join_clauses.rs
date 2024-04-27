use gatesim::*;

use std::cmp::Ord;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputEntryN<T> {
    NewIndex(T, bool),
    // Value(v, n): v - calculated value, n - original negation from this output entry
    Value(bool, bool),
}

// return true if further changes is needed.
// output_map includes circuit's inputs.
pub(crate) fn join_and_remove_clauses<T>(
    input_len: &mut usize,
    clauses: &mut Vec<(Clause<T>, bool)>,
    outputs: &[(T, bool)],
    output_map: &mut [OutputEntryN<T>],
    oim_opt: &mut Option<Vec<usize>>,
) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    if clauses.len() + *input_len == 0 {
        return false;
    }
    // println!("JoinAndRemove Start");
    let mut maxl = 0;
    for (c, _) in clauses.iter() {
        for (l, _) in &c.literals {
            maxl = std::cmp::max(usize::try_from(*l).unwrap(), maxl);
        }
    }
    maxl = std::cmp::max(maxl + 1, *input_len + clauses.len());

    let mut output_usages = vec![0; maxl];
    for (c, _) in clauses.iter() {
        for (l, _) in &c.literals {
            let l = usize::try_from(*l).unwrap();
            // if l >= output_usages.len() {
            //     println!("Failed xxx: {} {:?} {}", ci,
            //              c.literals.iter().map(|(ll,ln)| (usize::try_from(*ll).unwrap(), *ln))
            //              .collect::<Vec<_>>(),
            //              l);
            // }
            output_usages[l] += 1;
        }
    }
    for (o, _) in outputs.iter() {
        if let OutputEntryN::NewIndex(o, _) = output_map[usize::try_from(*o).unwrap()] {
            let o = usize::try_from(o).unwrap();
            output_usages[o] += 1;
        }
    }

    // generate orig_index_map - convert new indexes to old original indexes
    // include only first entries in output_map
    let oim: &mut Vec<usize> = if let Some(oim) = oim_opt {
        oim
    } else {
        let mut oim = vec![None; clauses.len() + *input_len];
        for (i, x) in output_map.iter().enumerate() {
            if let OutputEntryN::NewIndex(x, _) = x {
                let x = usize::try_from(*x).unwrap();
                if x < oim.len() && oim[x].is_none() {
                    oim[x] = Some(i);
                }
            }
        }
        // to real oim
        let oim = oim
            .into_iter()
            .map(|x| x.unwrap_or_default())
            .collect::<Vec<_>>();
        *oim_opt = Some(oim);
        oim_opt.as_mut().unwrap()
    };

    // traversing and join clauses
    #[derive(Clone, Copy, Debug)]
    struct StackEntry {
        node: usize,
        way: usize,
        clause_id: Option<usize>,
        negate_join: bool,
    }
    let mut visited = vec![false; clauses.len()];
    let mut visited_for_collect = vec![false; clauses.len()];
    // clauses length before second pass
    let mut clause_len_before_second = vec![0; clauses.len()];

    // collect output duplicates recognized by comparing output map entries
    let output_dups = {
        let mut output_dups = HashMap::<usize, Vec<(usize, bool)>>::new();
        for (o, _) in outputs.iter() {
            let o = usize::try_from(*o).unwrap();
            match output_map[o] {
                OutputEntryN::NewIndex(ni, omn) => {
                    let ni = usize::try_from(ni).unwrap();
                    if let Some(dups) = output_dups.get_mut(&ni) {
                        dups.push((o, omn));
                    } else {
                        output_dups.insert(ni, vec![(o, omn)]);
                    }
                }
                _ => (),
            }
        }
        output_dups.retain(|_, d| d.len() >= 2);
        output_dups
    };

    let mut do_next_iter = false;
    //
    // traverse 1: resolve one literal clauses and resolve other clauses
    //
    for (o, _) in outputs.iter() {
        let o = usize::try_from(*o).unwrap();
        // DEBUG
        // println!("Direct output: {}", o);
        // DEBUG
        if o < *input_len {
            continue;
        }
        let o = match output_map[o] {
            OutputEntryN::NewIndex(o, _) => {
                let o = usize::try_from(o).unwrap();
                if o < *input_len {
                    continue;
                }
                o
            }
            OutputEntryN::Value(_, _) => continue,
        };
        let mut stack = Vec::<StackEntry>::new();
        // DEBUG
        // println!("Converted output: {}", o);
        // DEBUG
        stack.push(StackEntry {
            node: o - *input_len,
            way: 0,
            clause_id: None,
            negate_join: false,
        });
        while !stack.is_empty() {
            let top = stack.last_mut().unwrap();
            let node_index = top.node;
            let (clause, clause_neg) = &clauses[node_index];

            if top.way == 0 {
                if let Some(clause_id) = top.clause_id {
                    // different visited masks for collection
                    if !visited_for_collect[node_index] {
                        if clause_id != node_index {
                            // make visited only for children to join
                            visited_for_collect[node_index] = true;
                        }
                    } else {
                        stack.pop();
                        continue;
                    }
                } else {
                    if !visited[node_index] {
                        visited[node_index] = true;
                    } else {
                        stack.pop();
                        continue;
                    }
                }
            }
            if top.way < clause.literals.len() {
                let way = top.way;
                top.way += 1;
                if let Some(clause_id) = top.clause_id {
                    let n = clause.literals[way].1;
                    let l = usize::try_from(clause.literals[way].0).unwrap();
                    if let OutputEntryN::NewIndex(l1, n1) = output_map[oim[l]] {
                        let l1_u = usize::try_from(l1).unwrap();
                        if l1_u >= *input_len {
                            let lclause = &clauses[l1_u - *input_len];
                            // if clause kind is same and if and-clause then
                            // must be negated literal finally
                            if lclause.0.kind == clause.kind
                                && (lclause.0.kind != ClauseKind::And
                                    || !(lclause.1 ^ n1 ^ n))
                                    // ignore clause with multiple usage
                                    && output_usages[l1_u] <= 1
                            {
                                // push with clause kind
                                stack.push(StackEntry {
                                    node: l1_u - *input_len,
                                    way: 0,
                                    clause_id: Some(clause_id),
                                    negate_join: lclause.1 ^ n1 ^ n,
                                });
                            }
                        }
                    }
                } else {
                    let l = usize::try_from(clause.literals[way].0).unwrap();
                    if l >= *input_len {
                        stack.push(StackEntry {
                            node: l - *input_len,
                            way: 0,
                            clause_id: None,
                            negate_join: false,
                        });
                    }
                }
            } else if let Some(clause_id) = top.clause_id {
                // resolving clause collecting
                if clause_id != node_index {
                    let literals_to_add = clause.literals.clone();
                    // put to target clause
                    let target_clause = &mut clauses[clause_id];
                    target_clause.0.literals.extend(literals_to_add);
                    // resolve negation: only for XOR clauses
                    target_clause.1 ^= top.negate_join;
                    {
                        let (clause, _) = &mut clauses[node_index];
                        clause.literals.clear();
                    }
                } else {
                    // remove literals of clauses
                    let mut to_remove = vec![];
                    for (li, (l, n)) in clause
                        .literals
                        .iter()
                        .enumerate()
                        .take(clause_len_before_second[node_index])
                    {
                        let l_u = usize::try_from(*l).unwrap();
                        if let OutputEntryN::NewIndex(l1, n1) = output_map[oim[l_u]] {
                            // check if can be merged
                            let l1_u = usize::try_from(l1).unwrap();
                            if l1_u >= *input_len {
                                let lclause = &clauses[l1_u - *input_len];
                                // if clause kind is same and if and-clause then
                                // must be negated literal finally
                                if lclause.0.kind == clause.kind
                                    && (lclause.0.kind != ClauseKind::And
                                        || !(lclause.1 ^ n1 ^ n))
                                        // ignore clause with multiple usage
                                        && output_usages[l1_u] <= 1
                                {
                                    to_remove.push(li);
                                }
                            }
                        }
                    }
                    let mut new_literals = vec![];
                    let mut j = 0;
                    // create new literals without removed literals
                    for (i, l) in clause.literals.iter().enumerate() {
                        if let Some(idx) = to_remove.get(j) {
                            if *idx == i {
                                j += 1;
                            } else {
                                new_literals.push(*l);
                            }
                        } else {
                            new_literals.push(*l);
                        }
                    }
                    clauses[node_index].0.literals = new_literals;
                }
                // repeat process??? really needed?? yes - translation.
                top.way = 0;
                top.clause_id = None;
            } else {
                // DEBUG
                // println!("ClauseToReduce: {}", *input_len + node_index);
                // DEBUG
                // resolve values and indexes for current clause
                let cur_out_n1 = if let OutputEntryN::NewIndex(_, n) =
                    output_map[oim[*input_len + node_index]]
                {
                    n
                } else {
                    panic!("Unexpected");
                };
                if clause.literals.is_empty() {
                    // DEBUG
                    // println!("ClauseToReduce empty: {} {}", *input_len + node_index,
                    //          oim[*input_len + node_index]);
                    // DEBUG
                    // fill up by zero ^ neg (and additional negation for And clause)
                    output_map[oim[*input_len + node_index]] = OutputEntryN::Value(
                        *clause_neg ^ cur_out_n1 ^ (clause.kind == ClauseKind::And),
                        cur_out_n1,
                    );
                    do_next_iter = true;
                } else if clause.literals.len() == 1 {
                    // propagate to output_map
                    let l = usize::try_from(clause.literals[0].0).unwrap();
                    // DEBUG
                    // println!("ClauseToReduce 1lit-clause: {}: {} {}", *input_len + node_index,
                    //          l, oim[l]);
                    // DEBUG
                    match output_map[oim[l]] {
                        OutputEntryN::NewIndex(x, n1) => {
                            // DEBUG
                            // println!("Reduce 1lit-clause lit: {} {}: {} {} {}",
                            //          *input_len + node_index,
                            //          oim[*input_len + node_index],
                            //          l,
                            //          oim[l],
                            //          usize::try_from(x).unwrap());
                            // DEBUG
                            output_map[oim[*input_len + node_index]] = OutputEntryN::NewIndex(
                                x,
                                cur_out_n1 ^ n1 ^ clause.literals[0].1 ^ *clause_neg,
                            );
                            // propagate usage of clause
                            output_usages[usize::try_from(x).unwrap()] +=
                                output_usages[*input_len + node_index] - 1;
                        }
                        OutputEntryN::Value(v, _) => {
                            // DEBUG
                            // println!("Reduce 1lit-clause v: {} {}: {} {} {} {}",
                            //          *input_len + node_index,
                            //          oim[*input_len + node_index],
                            //          cur_out_n1,
                            //          v,
                            //          clause.literals[0].1,
                            //          *clause_neg);
                            // DEBUG
                            output_map[oim[*input_len + node_index]] = OutputEntryN::Value(
                                cur_out_n1 ^ v ^ clause.literals[0].1 ^ *clause_neg,
                                cur_out_n1,
                            );
                        }
                    }
                    do_next_iter = true;
                } else {
                    // resolve clause
                    let mut new_literals = vec![];
                    let mut do_second_pass = false;
                    let mut neg_clause = false;
                    let mut clause_false = false;
                    for (l, n) in &clause.literals {
                        let l_u = usize::try_from(*l).unwrap();
                        match output_map[oim[l_u]] {
                            OutputEntryN::NewIndex(l1, n1) => {
                                // check if can be merged
                                if !do_second_pass {
                                    let l1_u = usize::try_from(l1).unwrap();
                                    if l1_u >= *input_len {
                                        let lclause = &clauses[l1_u - *input_len];
                                        // if clause kind is same and if and-clause then
                                        // must be negated literal finally
                                        if lclause.0.kind == clause.kind
                                            && (lclause.0.kind != ClauseKind::And
                                                || !(lclause.1 ^ n1 ^ n))
                                                // ignore clause with multiple usage
                                                && output_usages[l1_u] <= 1
                                        {
                                            // use second_pass for node to collect child clauses
                                            do_second_pass = true;
                                        }
                                    }
                                }
                                new_literals.push((*l, *n));
                            }
                            OutputEntryN::Value(v1, _) => {
                                let v = n ^ v1;
                                match clause.kind {
                                    ClauseKind::And => {
                                        if !v {
                                            new_literals.clear();
                                            // set clause to false (change to Xor clause)
                                            clause_false = true;
                                            break;
                                        }
                                    }
                                    ClauseKind::Xor => {
                                        neg_clause ^= v;
                                    }
                                }
                                do_next_iter = true;
                            }
                        }
                    }
                    {
                        let (clause, clause_neg) = &mut clauses[node_index];
                        clause.literals = new_literals;
                        *clause_neg ^= neg_clause;
                        if clause_false {
                            clause.kind = ClauseKind::Xor;
                        }
                        if clause.literals.len() >= 2 {
                            clause_len_before_second[node_index] = clause.literals.len();

                            if do_second_pass {
                                // prepare to second pass to collect clauses
                                //println!("Second pass: {:?}", clause);
                                top.way = 0; // reset way
                                top.clause_id = Some(node_index);
                                do_next_iter = true;
                                continue; // skip popping
                            } else {
                                // update same literals and negations for literals at end
                                for (l, n) in &mut clause.literals {
                                    let l_u = usize::try_from(*l).unwrap();
                                    if let OutputEntryN::NewIndex(l1, n1) = output_map[oim[l_u]] {
                                        *l = l1;
                                        *n ^= n1;
                                    }
                                }
                            }
                        } else if clause.literals.len() == 1 {
                            // propagate to output_map
                            let l = usize::try_from(clause.literals[0].0).unwrap();
                            match output_map[oim[l]] {
                                OutputEntryN::NewIndex(x, n1) => {
                                    output_map[oim[*input_len + node_index]] =
                                        OutputEntryN::NewIndex(
                                            x,
                                            cur_out_n1 ^ n1 ^ clause.literals[0].1 ^ *clause_neg,
                                        );
                                    // propagate usage of clause
                                    output_usages[usize::try_from(x).unwrap()] +=
                                        output_usages[*input_len + node_index] - 1;
                                }
                                OutputEntryN::Value(v, _) => {
                                    output_map[oim[*input_len + node_index]] = OutputEntryN::Value(
                                        cur_out_n1 ^ v ^ clause.literals[0].1 ^ *clause_neg,
                                        cur_out_n1,
                                    );
                                }
                            }
                            do_next_iter = true;
                        } else {
                            // resolve empty clause
                            // fill up by zero ^ neg (and additional negation for And clause)
                            output_map[oim[*input_len + node_index]] = OutputEntryN::Value(
                                *clause_neg ^ cur_out_n1 ^ (clause.kind == ClauseKind::And),
                                cur_out_n1,
                            );
                            do_next_iter = true;
                        }
                    }
                }
                stack.pop();
            }
        }
    }

    // before finding usages: just update output duplicate recognized by output_map entries
    for (ni, dups) in output_dups {
        if let Some((i2, _, oomn, ni2, nin2)) = dups
            .iter()
            .enumerate()
            .filter_map(|(i, (o, omn))| {
                if let OutputEntryN::NewIndex(ni2, n2) = output_map[*o] {
                    Some((i, *o, *omn, usize::try_from(ni2).unwrap(), n2))
                } else {
                    None
                }
            })
            .find(|(_, _, _, ni2, _)| *ni2 != ni)
        {
            // just update to other
            for (i, (o, omn)) in dups.iter().enumerate() {
                if i != i2 {
                    if let OutputEntryN::NewIndex(cni, _) = output_map[*o] {
                        let cni = usize::try_from(cni).unwrap();
                        if cni != ni2 {
                            // only if not updated
                            output_map[*o] = OutputEntryN::NewIndex(
                                T::try_from(ni2).unwrap(),
                                oomn ^ omn ^ nin2,
                            );
                        }
                    }
                }
            }
        }
    }

    let mut used_new_outputs = vec![false; *input_len + clauses.len()];
    //
    // traverse 2 - used_new_outputs - fill usage of outputs
    //
    for (o, _) in outputs.iter() {
        let o = usize::try_from(*o).unwrap();
        if o < *input_len {
            used_new_outputs[o] = true;
            continue;
        }
        let o = match output_map[o] {
            OutputEntryN::NewIndex(o, _) => {
                let o = usize::try_from(o).unwrap();
                if o < *input_len {
                    // println!("UsedNewOutputs1: {}", o);
                    used_new_outputs[o] = true;
                    continue;
                }
                o
            }
            OutputEntryN::Value(_, _) => continue,
        };
        let mut stack = Vec::<StackEntry>::new();
        stack.push(StackEntry {
            node: o - *input_len,
            way: 0,
            clause_id: None,
            negate_join: false,
        });
        while !stack.is_empty() {
            let top = stack.last_mut().unwrap();
            let node_index = top.node;
            let (clause, _) = &clauses[node_index];
            if top.way == 0 {
                if !used_new_outputs[*input_len + node_index] {
                    // println!("UsedNewOutputs2: {}", *input_len + node_index);
                    used_new_outputs[*input_len + node_index] = true;
                } else {
                    stack.pop();
                    continue;
                }
            }

            if top.way < clause.literals.len() {
                let way = top.way;
                top.way += 1;
                let l = usize::try_from(clause.literals[way].0).unwrap();
                if l >= *input_len {
                    stack.push(StackEntry {
                        node: l - *input_len,
                        way: 0,
                        clause_id: None,
                        negate_join: false,
                    });
                } else {
                    // println!("UsedNewOutputs3: {}", l);
                    used_new_outputs[l] = true;
                }
            } else {
                stack.pop();
            }
        }
    }

    // translate literals map - from previous to current index
    let old_len = *input_len + clauses.len();
    let old_trans_map = used_new_outputs
        .iter()
        .enumerate()
        .filter(|(_, x)| **x)
        .map(|(i, _)| T::try_from(i).unwrap())
        .collect::<Vec<_>>();
    let mut trans_map = vec![T::default(); *input_len + clauses.len()];
    for (i, x) in old_trans_map.iter().enumerate() {
        trans_map[usize::try_from(*x).unwrap()] = T::try_from(i).unwrap();
    }
    *clauses = used_new_outputs[*input_len..]
        .iter()
        .enumerate()
        .filter(|(_, x)| **x)
        .map(|(i, _)| clauses[i].clone())
        .collect::<Vec<_>>();
    // translate literal in clauses
    for (clause, _) in clauses.iter_mut() {
        for (l, n) in &mut clause.literals {
            let l_u = usize::try_from(*l).unwrap();
            if let OutputEntryN::NewIndex(l1, n1) = output_map[oim[l_u]] {
                *l = trans_map[usize::try_from(l1).unwrap()];
                *n ^= n1;
            }
        }
    }

    // output_to_skip_set - set of outputs (that are circuit outputs) to skip and
    // that outputs will be translated while processing outputs.
    let output_to_skip_set =
        HashSet::<usize>::from_iter(outputs.iter().map(|(x, _)| usize::try_from(*x).unwrap()));
    for j in 0..old_len {
        let oim_id = oim[j];
        if !output_to_skip_set.contains(&oim_id) {
            if let OutputEntryN::NewIndex(idx, n) = output_map[oim_id] {
                if *input_len != 0 && j < *input_len && !used_new_outputs[j] {
                    // CHECK
                    output_map[oim_id] = OutputEntryN::Value(n, n);
                } else {
                    output_map[oim_id] =
                        OutputEntryN::NewIndex(trans_map[usize::try_from(idx).unwrap()], n);
                }
            }
        }
    }

    *oim = used_new_outputs
        .iter()
        .enumerate()
        .filter(|(_, x)| **x)
        .map(|(i, _)| oim[i])
        .collect::<Vec<_>>();

    // translate outputs for circuit outputs
    // DEBUG
    let mut some_find = false;
    // !DEBUG
    let mut outputs_processed = HashSet::new();
    for (o, _) in outputs {
        let o = usize::try_from(*o).unwrap();
        if outputs_processed.contains(&o) {
            continue;
        }
        outputs_processed.insert(o);
        if let OutputEntryN::NewIndex(idx, n) = output_map[o] {
            let idx_u = usize::try_from(idx).unwrap();
            if *input_len != 0 && idx_u < *input_len && !used_new_outputs[idx_u] {
                // DEBUG
                // println!("JNR: Output: {} {}", idx_u, n);
                // DEBUG
                output_map[o] = OutputEntryN::Value(false ^ n, n);
            } else {
                let newidx = trans_map[usize::try_from(idx).unwrap()];
                let newidx_u = usize::try_from(newidx).unwrap();
                // DEBUG
                if !some_find {
                    // println!("Some find: {} {}", usize::try_from(idx).unwrap(), newidx_u);
                    some_find = true;
                }
                // println!("LastOutputMapNewIdx: {} {}", o, newidx_u);
                // !DEBUG
                match output_map[oim[newidx_u]] {
                    OutputEntryN::Value(v, orig_n) => {
                        output_map[o] = OutputEntryN::Value(v ^ n ^ orig_n, n);
                    }
                    OutputEntryN::NewIndex(_, _) => {
                        output_map[o] = OutputEntryN::NewIndex(newidx, n);
                    }
                }
            }
            // if *input_len != 0 && idx_u < *input_len && !used_new_outputs[idx_u] {
            //     output_map[o] = OutputEntryN::Value(false);
            // } else {
            //     output_map[o] = OutputEntryN::NewIndex(trans_map[usize::try_from(idx).unwrap()], n);
            // }
        }
    }
    // DEBUG
    // for (i, om) in output_map
    //     .iter()
    //     .map(|oe| match oe {
    //         OutputEntryN::NewIndex(v, n) => {
    //             OutputEntryN::NewIndex(usize::try_from(*v).unwrap(), *n)
    //         }
    //         OutputEntryN::Value(v, on) => OutputEntryN::Value(*v, *on),
    //     })
    //     .enumerate() {
    //     println!("OutputMap {} {:?}", i, om);
    // }
    // DEBUG
    *input_len = used_new_outputs[..*input_len]
        .iter()
        .enumerate()
        .filter(|(_, x)| **x)
        .map(|(i, _)| trans_map[i])
        .map(|x| usize::try_from(x).unwrap())
        .max()
        .map(|x| x + 1)
        .unwrap_or_default();
    do_next_iter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_and_remove_clauses() {
        // testcase
        // empty
        let mut input_len = 0;
        let mut clauses = vec![];
        let outputs: [(usize, bool); 0] = [];
        let mut oim_opt = None;
        let mut output_map: [OutputEntryN<usize>; 0] = [];
        assert!(!join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(0, input_len);
        assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
        assert_eq!(Vec::<OutputEntryN<usize>>::new(), output_map.to_vec());

        // testcase
        // trivial no changes
        let mut input_len = 3;
        let mut clauses = vec![(Clause::new_and([(0, false), (1, false), (2, false)]), false)];
        let outputs = [(3, false)];
        let mut oim_opt = None;
        let mut output_map = [
            OutputEntryN::NewIndex(0, false),
            OutputEntryN::NewIndex(1, false),
            OutputEntryN::NewIndex(2, false),
            OutputEntryN::NewIndex(3, false),
        ];
        assert!(!join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(3, input_len);
        assert_eq!(
            vec![(Clause::new_and([(0, false), (1, false), (2, false)]), false)],
            clauses
        );
        assert_eq!(
            [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
            ],
            output_map
        );

        // testcase
        // process empty clause
        for tv in 0..8 {
            let mut input_len = 0;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let xor = (tv & 4) != 0;
            let mut clauses = vec![(
                if xor {
                    Clause::new_xor([])
                } else {
                    Clause::new_and([])
                },
                t,
            )];
            let outputs = [(0, false)];
            let mut oim_opt = None;
            let mut output_map = [OutputEntryN::NewIndex(0, t1)];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!([OutputEntryN::Value(t ^ t1 ^ !xor, t1),], output_map);
        }

        // testcase
        // process resolving empty clause (->false) in parent clause
        for tv in 0..8 {
            let mut input_len = 2;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let xor = (tv & 4) != 0;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    !xor,
                ),
                (Clause::new_and([(0, false), (1, false), (2, false)]), t),
            ];
            let outputs = [(3, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(t ^ t1, t1),
                ],
                output_map
            );
        }

        // testcase
        // process resolving empty clause (->true) in parent clause
        for xor in [false, true] {
            let mut input_len = 2;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    true ^ !xor,
                ),
                (Clause::new_and([(0, false), (1, false), (2, false)]), false),
            ];
            let outputs = [(3, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(2, input_len);
            assert_eq!(
                vec![(Clause::new_and([(0, false), (1, false)]), false)],
                clauses
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::Value(true, false),
                    OutputEntryN::NewIndex(2, false),
                ],
                output_map
            );
        }

        // testcase
        // process resolving empty clause (->true) in parent clause
        for xor in [false, true] {
            let mut input_len = 2;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    true ^ !xor,
                ),
                (Clause::new_and([(0, false), (1, false), (2, false)]), false),
                (Clause::new_xor([(0, false), (3, false)]), false),
            ];
            let outputs = [(3, false), (4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(2, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), false),
                    (Clause::new_xor([(0, false), (2, false)]), false)
                ],
                clauses
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::Value(true, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map
            );
        }

        // testcase
        // if parent and clause is after deleting all literals - then must be true
        for xor in [false, true] {
            let mut input_len = 2;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    true ^ !xor,
                ),
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    true ^ !xor,
                ),
                (Clause::new_and([(2, false), (3, false)]), false),
            ];
            let outputs = [(4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(true, false),
                    OutputEntryN::Value(true, false),
                    OutputEntryN::Value(true, false),
                ],
                output_map
            );
        }

        // testcase
        // if parent xor clause is after deleting all literals - then must be false
        for xor in [false, true] {
            let mut input_len = 2;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    true ^ !xor,
                ),
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    true ^ !xor,
                ),
                (Clause::new_xor([(2, false), (3, false)]), false),
            ];
            let outputs = [(4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(true, false),
                    OutputEntryN::Value(true, false),
                    OutputEntryN::Value(false, false),
                ],
                output_map
            );
        }

        // testcase
        // process resolving empty clause (->t) in parent clause and change clause negation
        for tv in 0..16 {
            let mut input_len = 2;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let xor = (tv & 8) != 0;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    t ^ !xor,
                ),
                (Clause::new_xor([(0, false), (1, false), (2, false)]), t2),
            ];
            let outputs = [(3, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, t1),
                OutputEntryN::NewIndex(3, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(2, input_len);
            assert_eq!(
                vec![(Clause::new_xor([(0, false), (1, false)]), t ^ t1 ^ t2)],
                clauses
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::Value(t ^ t1, t1),
                    OutputEntryN::NewIndex(2, false),
                ],
                output_map
            );
        }

        // testcase
        // resolve output map for clause with one literal (including sign).
        for tv in 0..32 {
            let t0 = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let xor = (tv & 16) != 0;
            let mut input_len = 1;
            let mut clauses = vec![(
                if xor {
                    Clause::new_xor([(0, t0)])
                } else {
                    Clause::new_and([(0, t0)])
                },
                t1,
            )];
            let outputs = [(1, false)];
            let mut oim_opt = None;
            let mut output_map = [OutputEntryN::NewIndex(0, t2), OutputEntryN::NewIndex(1, t3)];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(1, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, t2),
                    OutputEntryN::NewIndex(0, t0 ^ t1 ^ t2 ^ t3),
                ],
                output_map
            );
        }

        // testcase
        // process resolving empty clause (->true) in parent clause to one-literal clause
        for tv in 0..32 {
            let t0 = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let xor = (tv & 16) != 0;
            let mut input_len = 1;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    true ^ !xor,
                ),
                (Clause::new_and([(0, t0), (1, false)]), t1),
            ];
            let outputs = [(2, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, t2),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, t3),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(1, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, t2),
                    OutputEntryN::Value(true, false),
                    OutputEntryN::NewIndex(0, t0 ^ t1 ^ t2 ^ t3),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // process resolving empty clause (->true) in parent clause to one-literal clause
        // with parent xor clause.
        for tv in 0..256 {
            let t0 = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let xor = (tv & 16) != 0;
            let t4 = (tv & 32) != 0;
            let t5 = (tv & 64) != 0;
            let t6 = (tv & 128) != 0;
            let mut input_len = 1;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    t5 ^ !xor,
                ),
                (Clause::new_xor([(0, t0), (1, t4)]), t1),
            ];
            let outputs = [(2, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, t2),
                OutputEntryN::NewIndex(1, t6),
                OutputEntryN::NewIndex(2, t3),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(1, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, t2),
                    OutputEntryN::Value(t5 ^ t6, t6),
                    OutputEntryN::NewIndex(0, t0 ^ t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // resolve output map for clause with one literal (including sign).
        for tv in 0..64 {
            let t0 = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let t4 = (tv & 16) != 0;
            let xor = (tv & 32) != 0;
            let mut input_len = 0;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    t0 ^ !xor,
                ),
                (Clause::new_xor([(0, t1)]), t2),
            ];
            let outputs = [(1, false)];
            let mut oim_opt = None;
            let mut output_map = [OutputEntryN::NewIndex(0, t3), OutputEntryN::NewIndex(1, t4)];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::Value(t0 ^ t3, t3),
                    OutputEntryN::Value(t0 ^ t1 ^ t2 ^ t3 ^ t4, t4),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause
        for tv in 0..4 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, t)]), false),
            ];
            let outputs = [(4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(Clause::new_and([(2, false), (0, false), (1, false)]), false)],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - different order
        for tv in 0..4 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(3, t), (2, false)]), false),
            ];
            let outputs = [(4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(Clause::new_and([(2, false), (0, false), (1, false)]), false)],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - different order 2
        for tv in 0..4 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(0, true), (3, t), (2, false)]), false),
            ];
            let outputs = [(4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(
                    Clause::new_and([(0, true), (2, false), (0, false), (1, false)]),
                    false
                )],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause
        for tv in 0..4 {
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut input_len = 3;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, true ^ t)]), false),
            ];
            let outputs = [(4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(!join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                    (Clause::new_and([(2, false), (3, true ^ t)]), false),
                ],
                clauses
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(4, false),
                ],
                output_map
            );
        }

        // testcase
        // do not join clause - some clause used by output
        for tv in 0..4 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, t)]), false),
            ];
            let outputs = [(3, false), (4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(!join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                    (Clause::new_and([(2, false), (3, t)]), false),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(4, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause 2 - xor clauses
        for tv in 0..8 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let mut clauses = vec![
                (Clause::new_xor([(0, false), (1, false)]), t),
                (Clause::new_xor([(2, false), (3, t1)]), false),
            ];
            let outputs = [(4, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t2),
                OutputEntryN::NewIndex(4, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(
                    Clause::new_xor([(2, false), (0, false), (1, false)]),
                    t ^ t1 ^ t2
                )],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t2),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause 2 - shared clause
        for tv in 0..4 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, t)]), false),
                (Clause::new_and([(1, false), (3, t), (4, true)]), false),
            ];
            let outputs = [(5, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
            ];
            assert!(!join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                    (Clause::new_and([(2, false), (3, t)]), false),
                    (Clause::new_and([(1, false), (3, t), (4, true)]), false),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(5, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - some literal is false
        for tv in 0..4 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_xor([]), false),
                (Clause::new_and([(2, false), (3, t), (4, false)]), false),
            ];
            let outputs = [(5, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses, "{}", tv);
            assert_eq!(
                [
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // with one-literal clause
        // testcase
        // join clause - with one literal clause
        for tv in 0..16 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_and([(2, false), (4, t)]), false),
            ];
            let outputs = [(5, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(Clause::new_and([(2, false), (0, false), (1, false)]), false)],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause
        for tv in 0..16 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_and([(2, false), (4, true ^ t)]), false),
            ];
            let outputs = [(5, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                    (
                        Clause::new_and([(2, false), (3, true ^ t ^ t2 ^ t3)]),
                        false
                    ),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(4, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause - some clause use by output
        for tv in 0..32 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let output = if (tv & 16) != 0 { 4 } else { 3 };
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_and([(2, false), (4, t)]), false),
            ];
            let outputs = [(output, false), (5, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                    (Clause::new_and([(2, false), (3, t ^ t2 ^ t3)]), false),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(4, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - with one literal clause
        for tv in 0..64 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let t4 = (tv & 16) != 0;
            let t5 = (tv & 32) != 0;
            let mut clauses = vec![
                (Clause::new_xor([(0, false), (1, false)]), t),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_xor([(2, false), (4, t1)]), false),
            ];
            let outputs = [(5, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t4),
                OutputEntryN::NewIndex(4, t5),
                OutputEntryN::NewIndex(5, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(
                    Clause::new_xor([(2, false), (0, false), (1, false)]),
                    t ^ t1 ^ t2 ^ t3 ^ t4 ^ t5
                )],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t4),
                    OutputEntryN::NewIndex(0, t2 ^ t3 ^ t4 ^ t5),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause - shared clause
        for tv in 0..16 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_and([(2, false), (4, t)]), false),
                (Clause::new_and([(1, false), (4, t), (5, true)]), false),
            ];
            let outputs = [(6, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                    (Clause::new_and([(2, false), (3, t ^ t2 ^ t3)]), false),
                    (
                        Clause::new_and([(1, false), (3, t ^ t2 ^ t3), (4, true)]),
                        false
                    ),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(5, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause - shared clause
        for tv in 0..16 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_and([(2, false), (4, t)]), false),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_and([(1, false), (6, t), (5, true)]), false),
            ];
            let outputs = [(7, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                    (Clause::new_and([(2, false), (3, t ^ t2 ^ t3)]), false),
                    (
                        Clause::new_and([(1, false), (3, t ^ t2 ^ t3), (4, true)]),
                        false
                    ),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(5, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause - shared clause
        for tv in 0..16 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_xor([]), false),
                (Clause::new_xor([(3, t2), (4, false)]), t3),
                (Clause::new_and([(2, false), (5, t)]), false),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_and([(1, false), (7, t), (6, true)]), false),
            ];
            let outputs = [(8, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(8, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                    (Clause::new_and([(2, false), (3, t ^ t2 ^ t3)]), false),
                    (
                        Clause::new_and([(1, false), (3, t ^ t2 ^ t3), (4, true)]),
                        false
                    ),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3),
                    OutputEntryN::NewIndex(5, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause - some literal is false
        for tv in 0..16 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_xor([]), false),
                (Clause::new_and([(2, false), (4, t), (5, false)]), false),
            ];
            let outputs = [(6, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses, "{}", tv);
            assert_eq!(
                [
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause - some literal is false - 2
        for tv in 0..16 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1 ^ t2 ^ t3),
                (Clause::new_and([]), true),
                (Clause::new_xor([(3, t2), (4, false)]), t3),
                (Clause::new_and([]), true),
                (Clause::new_and([(2, false), (5, t), (6, false)]), false),
            ];
            let outputs = [(7, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses, "{}", tv);
            assert_eq!(
                [
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::Value(false, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - with one literal clause - 2-clause chain
        for tv in 0..128 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let t4 = (tv & 16) != 0;
            let t5 = (tv & 32) != 0;
            let t6 = (tv & 64) != 0;
            let mut clauses = vec![
                (
                    Clause::new_and([(0, false), (1, false)]),
                    t ^ t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6,
                ),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_xor([(4, t5)]), t6),
                (Clause::new_and([(2, false), (5, t)]), false),
            ];
            let outputs = [(6, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(4, t4),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(Clause::new_and([(2, false), (0, false), (1, false)]), false)],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3 ^ t4),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - different order - many joined clauses
        for tv in 0..4 {
            let mut input_len = 8;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, false)]), t ^ t1),
                (
                    Clause::new_and([
                        (4, false),
                        (8, t),
                        (5, false),
                        (9, t),
                        (6, false),
                        (7, false),
                    ]),
                    false,
                ),
            ];
            let outputs = [(10, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(8, t1),
                OutputEntryN::NewIndex(9, t1),
                OutputEntryN::NewIndex(10, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(8, input_len);
            assert_eq!(
                vec![(
                    Clause::new_and([
                        (4, false),
                        (5, false),
                        (6, false),
                        (7, false),
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false)
                    ]),
                    false
                )],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, false),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(5, false),
                    OutputEntryN::NewIndex(6, false),
                    OutputEntryN::NewIndex(7, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(8, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - different order - many joined clauses - and omit one input
        for tv in 0..4 {
            let mut input_len = 8;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, false)]), t ^ t1),
                (
                    Clause::new_and([(4, false), (8, t), (6, false), (9, t), (7, false)]),
                    false,
                ),
            ];
            let outputs = [(10, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(8, t1),
                OutputEntryN::NewIndex(9, t1),
                OutputEntryN::NewIndex(10, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(7, input_len);
            assert_eq!(
                vec![(
                    Clause::new_and([
                        (4, false),
                        (5, false),
                        (6, false),
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false)
                    ]),
                    false
                )],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, false),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(5, false),
                    OutputEntryN::NewIndex(6, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(7, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - different order - many joined clauses.
        // previously reduced clauses and sparse output_map.
        for tv in 0..4 {
            let mut input_len = 8;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, false)]), t ^ t1),
                (
                    Clause::new_and([
                        (4, false),
                        (8, t),
                        (5, false),
                        (9, t),
                        (6, false),
                        (7, false),
                    ]),
                    false,
                ),
            ];
            let outputs = [(13, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(8, t1),
                OutputEntryN::NewIndex(9, t1),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(10, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(8, input_len);
            assert_eq!(
                vec![(
                    Clause::new_and([
                        (4, false),
                        (5, false),
                        (6, false),
                        (7, false),
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false)
                    ]),
                    false
                )],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(3, false),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(5, false),
                    OutputEntryN::NewIndex(6, false),
                    OutputEntryN::NewIndex(7, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(8, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - different order - many joined clauses.
        // previously reduced clauses and sparse output_map - different indexes in output_map.
        for tv in 0..4 {
            let mut input_len = 8;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (1, false)]), t ^ t1),
                (Clause::new_and([(2, false), (3, false)]), t ^ t1),
                (
                    Clause::new_and([
                        (4, false),
                        (8, t),
                        (5, false),
                        (9, t),
                        (6, false),
                        (7, false),
                    ]),
                    false,
                ),
            ];
            let outputs = [(13, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(8, t1),
                OutputEntryN::NewIndex(9, t1),
                OutputEntryN::NewIndex(8, false),
                OutputEntryN::NewIndex(10, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(8, input_len);
            assert_eq!(
                vec![(
                    Clause::new_and([
                        (4, false),
                        (5, false),
                        (6, false),
                        (7, false),
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false)
                    ]),
                    false
                )],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(3, false),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(5, false),
                    OutputEntryN::NewIndex(6, false),
                    OutputEntryN::NewIndex(7, false),
                    OutputEntryN::NewIndex(5, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(8, false), // ???
                    OutputEntryN::NewIndex(8, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // join clause - with one literal clause - 2-clause chain
        // reduced sparse output_map
        for tv in 0..128 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let t4 = (tv & 16) != 0;
            let t5 = (tv & 32) != 0;
            let t6 = (tv & 64) != 0;
            let mut clauses = vec![
                (
                    Clause::new_and([(0, false), (1, false)]),
                    t ^ t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6,
                ),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_xor([(4, t5)]), t6),
                (Clause::new_and([(2, false), (5, t)]), false),
            ];
            let outputs = [(9, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(4, t4),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(6, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![(Clause::new_and([(2, false), (0, false), (1, false)]), false)],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3 ^ t4),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(3, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // do not join clause - with one literal clause - 2-clause chain
        // reduced sparse output_map
        for tv in 0..128 {
            let mut input_len = 3;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let t3 = (tv & 8) != 0;
            let t4 = (tv & 16) != 0;
            let t5 = (tv & 32) != 0;
            let t6 = (tv & 64) != 0;
            let mut clauses = vec![
                (
                    Clause::new_and([(0, false), (1, false)]),
                    t ^ t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6,
                ),
                (Clause::new_xor([(3, t2)]), t3),
                (Clause::new_xor([(4, t5)]), t6),
                (Clause::new_and([(2, false), (5, t)]), false),
                (Clause::new_xor([(0, false), (2, false), (4, t)]), false),
            ];
            let outputs = [(9, false), (10, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, t1),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(4, t4),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(3, input_len);
            assert_eq!(
                vec![
                    (
                        Clause::new_and([(0, false), (1, false)]),
                        t ^ t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6,
                    ),
                    (
                        Clause::new_and([(2, false), (3, t ^ t2 ^ t3 ^ t4 ^ t5 ^ t6)]),
                        false
                    ),
                    (
                        Clause::new_xor([(0, false), (2, false), (3, t ^ t2 ^ t3 ^ t4)]),
                        false
                    ),
                ],
                clauses,
                "{}",
                tv
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, t1),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3 ^ t4),
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(5, false),
                ],
                output_map,
                "{}",
                tv
            );
        }

        // testcase
        // process resolving empty clause (->t) in parent clause and change clause negation
        // reduced sparse output_map
        for tv in 0..16 {
            let mut input_len = 2;
            let t = (tv & 1) != 0;
            let t1 = (tv & 2) != 0;
            let t2 = (tv & 4) != 0;
            let xor = (tv & 8) != 0;
            let mut clauses = vec![
                (
                    if xor {
                        Clause::new_xor([])
                    } else {
                        Clause::new_and([])
                    },
                    t ^ !xor,
                ),
                (Clause::new_xor([(0, false), (1, false), (2, false)]), t2),
            ];
            let outputs = [(6, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(2, t1),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(3, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(2, input_len);
            assert_eq!(
                vec![(Clause::new_xor([(0, false), (1, false)]), t ^ t1 ^ t2)],
                clauses
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::Value(false, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::Value(t ^ t1, t1),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(2, false),
                ],
                output_map
            );
        }

        // testcase
        let mut input_len = 6;
        let mut clauses = vec![
            (Clause::new_and([(0, false), (1, false)]), false),
            (Clause::new_xor([(1, false), (2, false)]), false),
            (Clause::new_and([(3, false), (4, false)]), true),
            (Clause::new_xor([(3, false), (5, false)]), true),
            (Clause::new_and([(6, false), (7, false), (8, false)]), false),
            (Clause::new_xor([(6, false), (7, true), (9, false)]), false),
            (Clause::new_and([(10, true), (11, false)]), false),
        ];
        let outputs = [(12, false)];
        let mut oim_opt = None;
        let mut output_map = [
            OutputEntryN::NewIndex(0, false),
            OutputEntryN::NewIndex(1, false),
            OutputEntryN::NewIndex(2, false),
            OutputEntryN::NewIndex(3, false),
            OutputEntryN::NewIndex(4, false),
            OutputEntryN::NewIndex(5, false),
            OutputEntryN::NewIndex(6, false),
            OutputEntryN::NewIndex(7, false),
            OutputEntryN::NewIndex(8, false),
            OutputEntryN::NewIndex(9, false),
            OutputEntryN::NewIndex(10, false),
            OutputEntryN::NewIndex(11, false),
            OutputEntryN::NewIndex(12, false),
        ];
        assert!(join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(6, input_len);
        assert_eq!(
            vec![
                (Clause::new_and([(0, false), (1, false)]), false),
                (Clause::new_xor([(1, false), (2, false)]), false),
                (Clause::new_and([(3, false), (4, false)]), true),
                (Clause::new_and([(6, false), (7, false), (8, false)]), false),
                (
                    Clause::new_xor([(6, false), (7, true), (3, false), (5, false)]),
                    true
                ),
                (Clause::new_and([(9, true), (10, false)]), false),
            ],
            clauses
        );
        assert_eq!(
            [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(8, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(9, false),
                OutputEntryN::NewIndex(10, false),
                OutputEntryN::NewIndex(11, false),
            ],
            output_map,
        );

        // testcase
        let mut input_len = 6;
        let mut clauses = vec![
            (Clause::new_and([(0, false), (1, false)]), false),
            (Clause::new_xor([(1, false), (2, false)]), false),
            (Clause::new_and([(3, false), (4, false)]), true),
            (Clause::new_xor([(3, false), (5, false)]), true),
            (Clause::new_and([(6, false), (7, false), (8, false)]), false),
            (Clause::new_xor([(7, true), (9, false)]), false),
            (Clause::new_and([(10, true), (11, false)]), false),
        ];
        let outputs = [(12, false)];
        let mut oim_opt = None;
        let mut output_map = [
            OutputEntryN::NewIndex(0, false),
            OutputEntryN::NewIndex(1, false),
            OutputEntryN::NewIndex(2, false),
            OutputEntryN::NewIndex(3, false),
            OutputEntryN::NewIndex(4, false),
            OutputEntryN::NewIndex(5, false),
            OutputEntryN::NewIndex(6, false),
            OutputEntryN::NewIndex(7, false),
            OutputEntryN::NewIndex(8, false),
            OutputEntryN::NewIndex(9, false),
            OutputEntryN::NewIndex(10, false),
            OutputEntryN::NewIndex(11, false),
            OutputEntryN::NewIndex(12, false),
        ];
        assert!(join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(6, input_len);
        assert_eq!(
            vec![
                (Clause::new_xor([(1, false), (2, false)]), false),
                (Clause::new_and([(3, false), (4, false)]), true),
                (
                    Clause::new_and([(6, false), (7, false), (0, false), (1, false)]),
                    false
                ),
                (Clause::new_xor([(6, true), (3, false), (5, false)]), true),
                (Clause::new_and([(8, true), (9, false)]), false),
            ],
            clauses
        );
        assert_eq!(
            [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(8, false),
                OutputEntryN::NewIndex(9, false),
                OutputEntryN::NewIndex(10, false),
            ],
            output_map,
        );

        // testcase
        let mut input_len = 6;
        let mut clauses = vec![
            (Clause::new_and([(0, false), (1, false)]), false),
            (Clause::new_xor([(1, false), (2, false)]), false),
            (Clause::new_and([(3, false), (4, false)]), true),
            (Clause::new_xor([(3, false), (5, false)]), true),
            (Clause::new_xor([]), false),
            (
                Clause::new_and([(6, false), (7, false), (8, false), (10, false)]),
                false,
            ),
            (Clause::new_xor([(7, true), (9, false), (10, false)]), false),
            (Clause::new_and([(11, true), (12, false)]), false),
        ];
        let outputs = [(13, false)];
        let mut oim_opt = None;
        let mut output_map = [
            OutputEntryN::NewIndex(0, false),
            OutputEntryN::NewIndex(1, false),
            OutputEntryN::NewIndex(2, false),
            OutputEntryN::NewIndex(3, false),
            OutputEntryN::NewIndex(4, false),
            OutputEntryN::NewIndex(5, false),
            OutputEntryN::NewIndex(6, false),
            OutputEntryN::NewIndex(7, false),
            OutputEntryN::NewIndex(8, false),
            OutputEntryN::NewIndex(9, false),
            OutputEntryN::NewIndex(10, false),
            OutputEntryN::NewIndex(11, false),
            OutputEntryN::NewIndex(12, false),
            OutputEntryN::NewIndex(13, false),
        ];
        assert!(join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(4, input_len);
        assert_eq!(
            vec![
                //(Clause::new_and([(0, false), (1, false)]), false),
                (Clause::new_xor([(0, false), (1, false)]), false),
                //(Clause::new_and([(3, false), (4, false)]), true),
                //(Clause::new_xor([(2, false), (3, false)]), true),
                //(Clause::new_and([]), false),
                //(Clause::new_and([(6, false), (7, false), (8, false), (10, false)]), false),
                // clause 4 not join because earlier has been usage counted by deleted previous
                // clause
                (Clause::new_xor([(4, true), (2, false), (3, false)]), true),
                // make to one-literal clause (true, clause-5) -> clause-5.
                //(Clause::new_and([(11, true), (12, false)]), false),
            ],
            clauses
        );
        assert_eq!(
            [
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(5, false), // from one-literal clause
            ],
            output_map,
        );

        assert!(join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(4, input_len);
        assert_eq!(
            vec![(
                Clause::new_xor([(2, false), (3, false), (0, false), (1, false)]),
                false
            ),],
            clauses
        );
        assert_eq!(
            [
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(4, false),
            ],
            output_map,
        );

        // testcase
        let mut input_len = 6;
        let mut clauses = vec![
            (Clause::new_and([(0, false), (1, false)]), false),
            (Clause::new_xor([(1, false), (2, false)]), false),
            (Clause::new_and([(3, false), (4, false)]), true),
            (Clause::new_xor([(3, false), (5, false)]), true),
            (Clause::new_xor([(6, false)]), false),
            (Clause::new_xor([(9, false)]), false),
            (Clause::new_and([(10, false), (8, false)]), false),
            (Clause::new_xor([(7, true), (11, false)]), false),
            (Clause::new_and([(12, true), (13, false)]), false),
        ];
        let outputs = [(14, false)];
        let mut oim_opt = None;
        let mut output_map = [
            OutputEntryN::NewIndex(0, false),
            OutputEntryN::NewIndex(1, false),
            OutputEntryN::NewIndex(2, false),
            OutputEntryN::NewIndex(3, false),
            OutputEntryN::NewIndex(4, false),
            OutputEntryN::NewIndex(5, false),
            OutputEntryN::NewIndex(6, false),
            OutputEntryN::NewIndex(7, false),
            OutputEntryN::NewIndex(8, false),
            OutputEntryN::NewIndex(9, false),
            OutputEntryN::NewIndex(10, false),
            OutputEntryN::NewIndex(11, false),
            OutputEntryN::NewIndex(12, false),
            OutputEntryN::NewIndex(13, false),
            OutputEntryN::NewIndex(14, false),
        ];
        assert!(join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(6, input_len);
        assert_eq!(
            vec![
                (Clause::new_and([(3, false), (4, false)]), true),
                (Clause::new_and([(6, false), (0, false), (1, false)]), false),
                (
                    Clause::new_xor([(1, false), (2, false), (3, false), (5, false)]),
                    false
                ),
                (Clause::new_and([(7, true), (8, false)]), false),
            ],
            clauses
        );
        assert_eq!(
            [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(8, false),
                OutputEntryN::NewIndex(9, false),
            ],
            output_map,
        );

        // testcase
        for t in [false, true] {
            let mut input_len = 6;
            let mut clauses = vec![
                (Clause::new_and([(0, false), (2, false)]), false),
                (Clause::new_xor([(0, false), (2, false)]), false),
                (Clause::new_and([(3, false), (4, false)]), true),
                (Clause::new_xor([(3, false), (5, false)]), true),
                (Clause::new_xor([(6, false)]), false),
                (Clause::new_xor([(9, false)]), false),
                (Clause::new_and([(10, false), (8, false)]), false),
                (Clause::new_xor([(7, true), (11, false)]), false),
                (Clause::new_and([(12, true), (13, false)]), false),
            ];
            let outputs = [(1, false), (if t { 10 } else { 6 }, false), (14, false)];
            let mut oim_opt = None;
            let mut output_map = [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(3, false),
                OutputEntryN::NewIndex(4, false),
                OutputEntryN::NewIndex(5, false),
                OutputEntryN::NewIndex(6, false),
                OutputEntryN::NewIndex(7, false),
                OutputEntryN::NewIndex(8, false),
                OutputEntryN::NewIndex(9, false),
                OutputEntryN::NewIndex(10, false),
                OutputEntryN::NewIndex(11, false),
                OutputEntryN::NewIndex(12, false),
                OutputEntryN::NewIndex(13, false),
                OutputEntryN::NewIndex(14, false),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map,
                &mut oim_opt
            ));
            assert_eq!(6, input_len);
            assert_eq!(
                vec![
                    (Clause::new_and([(0, false), (2, false)]), false),
                    (Clause::new_and([(3, false), (4, false)]), true),
                    (Clause::new_and([(6, false), (7, false)]), false),
                    (
                        Clause::new_xor([(0, false), (2, false), (3, false), (5, false)]),
                        false
                    ),
                    (Clause::new_and([(8, true), (9, false)]), false),
                ],
                clauses
            );
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(1, false),
                    OutputEntryN::NewIndex(2, false),
                    OutputEntryN::NewIndex(3, false),
                    OutputEntryN::NewIndex(4, false),
                    OutputEntryN::NewIndex(5, false),
                    OutputEntryN::NewIndex(6, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(7, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(6, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(8, false),
                    OutputEntryN::NewIndex(9, false),
                    OutputEntryN::NewIndex(10, false),
                ],
                output_map,
            );
        }

        // testcase
        // chain of joining of clauses: were second resolved later.
        // testcase for bug when target join clause will be visited and can't be
        // joined later.
        let mut input_len = 3;
        let mut clauses = vec![
            (Clause::new_and([(0, false), (1, false)]), false),
            (Clause::new_and([(2, false), (3, false)]), false),
            (Clause::new_xor([]), false),
            (Clause::new_and([(4, true), (5, true)]), false), // 6:and(!4,!false)->!4
            (Clause::new_and([(0, true), (6, true)]), false), // to join: and(!0,!6)
        ];
        let outputs = [(7, false)];
        let mut oim_opt = None;
        let mut output_map = [
            OutputEntryN::NewIndex(0, false),
            OutputEntryN::NewIndex(1, false),
            OutputEntryN::NewIndex(2, false),
            OutputEntryN::NewIndex(3, false),
            OutputEntryN::NewIndex(4, false),
            OutputEntryN::NewIndex(5, false),
            OutputEntryN::NewIndex(6, false),
            OutputEntryN::NewIndex(7, false),
        ];
        assert!(join_and_remove_clauses(
            &mut input_len,
            &mut clauses,
            &outputs,
            &mut output_map,
            &mut oim_opt
        ));
        assert_eq!(3, input_len);
        assert_eq!(
            vec![(
                Clause::new_and([(0, true), (2, false), (0, false), (1, false)]),
                false
            )],
            clauses
        );
        assert_eq!(
            [
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::NewIndex(0, false),
                OutputEntryN::Value(false, false),
                OutputEntryN::NewIndex(0, true),
                OutputEntryN::NewIndex(3, false),
            ],
            output_map
        );
    }
}
