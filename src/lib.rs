use gatesim::*;

use std::cmp::Ord;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter;

pub fn deduplicate<T: Clone + Copy + Ord + PartialEq + Eq>(circuit: Circuit<T>) -> Circuit<T>
where
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
    T: Hash,
{
    let mut gate_map = HashMap::<Gate<T>, T>::new();
    let mut new_gates: Vec<Gate<T>> = vec![];
    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut gate_count = input_len;
    let mut output_map = Vec::from_iter(
        (0..input_len)
            .map(|x| T::try_from(x).unwrap())
            .chain(iter::repeat(T::default()).take(circuit.len())),
    );

    for (i, g) in circuit.gates().into_iter().enumerate() {
        let oi = input_len + i;
        let gi0 = output_map[usize::try_from(g.i0).unwrap()];
        let gi1 = output_map[usize::try_from(g.i1).unwrap()];
        // convert to new gate - ordered inputs if not nimpl.
        let (gi0, gi1) = if g.func != GateFunc::Nimpl && gi0 > gi1 {
            (gi1, gi0)
        } else {
            (gi0, gi1)
        };
        let newg = Gate {
            i0: gi0,
            i1: gi1,
            func: g.func,
        };
        if let Some(gindex) = gate_map.get(&newg) {
            // if found gate - then store its index into output_map
            output_map[oi] = *gindex;
        } else {
            // otherwise push to new_gates and to gate_map
            new_gates.push(newg);
            let gate_count_t = T::try_from(gate_count).unwrap();
            output_map[oi] = gate_count_t;
            gate_map.insert(newg, gate_count_t);
            gate_count += 1;
        }
    }

    let new_outputs = circuit
        .outputs()
        .into_iter()
        .map(|(x, n)| (output_map[usize::try_from(*x).unwrap()], *n))
        .collect::<Vec<_>>();

    Circuit::new(circuit.input_len(), new_gates, new_outputs).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEntry<T> {
    NewIndex(T),
    Value(bool),
}

// return circuit with assignment and mapping from older input to new input
// and output mapping from older output index to new output index or value
pub fn assign_to_circuit<T>(
    circuit: &Circuit<T>,
    inputs: impl IntoIterator<Item = (T, bool)>,
) -> (Circuit<T>, Vec<OutputEntry<T>>, Vec<OutputEntry<T>>)
where
    T: Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord,
    T: TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let len = circuit.len();

    let mut gate_map = vec![OutputEntry::Value(false); input_len + len];
    let mut rest_map = vec![true; input_len];
    // filter inputs
    for (g, v) in inputs.into_iter() {
        let g_u = usize::try_from(g).unwrap();
        rest_map[g_u] = false;
        gate_map[g_u] = OutputEntry::Value(v);
    }
    // generate output inputs
    let out_inputs = rest_map[0..input_len]
        .iter()
        .enumerate()
        .filter_map(|(i, x)| {
            if *x {
                Some(T::try_from(i).unwrap())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // make to_new_rest_map - conversion to new outputs
    for (i, j) in out_inputs.iter().enumerate() {
        gate_map[usize::try_from(*j).unwrap()] = OutputEntry::NewIndex(T::try_from(i).unwrap());
    }
    let new_input_len = out_inputs.len();
    let mut new_gates: Vec<Gate<T>> = vec![];

    let mut oi = new_input_len;
    for (i, g) in circuit.gates().into_iter().enumerate() {
        let ii = input_len + i;
        let gi0 = usize::try_from(g.i0).unwrap();
        let gi1 = usize::try_from(g.i1).unwrap();
        match gate_map[gi0] {
            OutputEntry::NewIndex(ni0) => {
                match gate_map[gi1] {
                    OutputEntry::NewIndex(ni1) => {
                        gate_map[ii] = OutputEntry::NewIndex(T::try_from(oi).unwrap());
                        new_gates.push(Gate {
                            i0: ni0,
                            i1: ni1,
                            func: g.func,
                        });
                        oi += 1;
                    }
                    OutputEntry::Value(v1) => {
                        gate_map[ii] = OutputEntry::NewIndex(T::try_from(oi).unwrap());
                        let vv0 = g.eval_args(false, v1);
                        let vv1 = g.eval_args(true, v1);
                        new_gates.push(Gate {
                            i0: ni0,
                            i1: ni0,
                            func: if !vv0 && vv1 {
                                // x
                                GateFunc::And
                            } else if vv0 && !vv1 {
                                // !x
                                GateFunc::Nor
                            } else if !vv0 && !vv1 {
                                // 0
                                GateFunc::Nimpl
                            } else {
                                panic!("Unexpected case!");
                            },
                        });
                        oi += 1;
                    }
                }
            }
            OutputEntry::Value(v0) => {
                match gate_map[gi1] {
                    OutputEntry::NewIndex(ni1) => {
                        gate_map[ii] = OutputEntry::NewIndex(T::try_from(oi).unwrap());
                        let vv0 = g.eval_args(v0, false);
                        let vv1 = g.eval_args(v0, true);
                        new_gates.push(Gate {
                            i0: ni1,
                            i1: ni1,
                            func: if !vv0 && vv1 {
                                // x
                                GateFunc::And
                            } else if vv0 && !vv1 {
                                // !x
                                GateFunc::Nor
                            } else if !vv0 && !vv1 {
                                // 0
                                GateFunc::Nimpl
                            } else {
                                panic!("Unexpected case!");
                            },
                        });
                        oi += 1;
                    }
                    OutputEntry::Value(v1) => {
                        let out = g.eval_args(v0, v1);
                        gate_map[ii] = OutputEntry::Value(out);
                    }
                }
            }
        }
    }

    // outputs
    let mut new_outputs = vec![];
    let mut output_entries = vec![];
    for (o, n) in circuit.outputs().iter() {
        let o_u = usize::try_from(*o).unwrap();
        match gate_map[o_u] {
            OutputEntry::NewIndex(no) => {
                output_entries.push(OutputEntry::NewIndex(
                    T::try_from(new_outputs.len()).unwrap(),
                ));
                new_outputs.push((no, *n));
            }
            OutputEntry::Value(v) => {
                output_entries.push(OutputEntry::Value(v ^ n));
            }
        }
    }

    (
        Circuit::<T>::new(T::try_from(new_input_len).unwrap(), new_gates, new_outputs).unwrap(),
        gate_map[0..input_len].to_vec(),
        output_entries,
    )
}

// reduce chain clause - one-literal-clause - clause.
// check whether all usages of clause only in other clause.
// reduce clauses to zero or ones (constants).
// remove duplicated literals in clause.
// reduce literals in clause.
// deduplication based on evaluation (evaluated values for all input values) (optional).
// xor detection in and-or and or-and clause tree.
// find common parts of clauses to reuse more parts.

fn reduce_clauses<T>(clauses: &mut [(Clause<T>, bool)]) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut to_reduce_tree = false;
    for (clause, cs) in clauses {
        clause.literals.sort();
        let old_len = clause.len();
        match clause.kind {
            ClauseKind::And => {
                clause.literals.dedup();
                let mut pl = None;
                let mut zero = false;
                for (l, _) in &clause.literals {
                    if let Some(pl) = pl {
                        if pl == l {
                            // we have l and not(l) -> clause = 0
                            zero = true;
                            break;
                        }
                    }
                    pl = Some(l);
                }
                if zero {
                    // IMPORTANT: empty clauses treat as false.
                    clause.literals.clear();
                }
            }
            ClauseKind::Xor => {
                let mut pl = None;
                let mut new_literals = vec![];

                for (l, s) in &clause.literals {
                    if *s {
                        *cs = !*cs;
                    }
                    if let Some(pl) = pl {
                        if pl == l {
                            // we have l and l -> reduce 0
                            new_literals.pop();
                        } else {
                            new_literals.push((*l, false));
                        }
                    } else {
                        new_literals.push((*l, false));
                    }
                    pl = Some(l);
                }
                clause.literals = new_literals;
            }
        }
        if old_len >= 2 && clause.len() < 2 {
            to_reduce_tree = true;
        }
    }
    to_reduce_tree
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEntryN<T> {
    NewIndex(T, bool),
    Value(bool),
}

// return true if further changes is needed.
// output_map includes circuit's inputs.
fn join_and_remove_clauses<T>(
    input_len: &mut usize,
    clauses: &mut Vec<(Clause<T>, bool)>,
    outputs: &[(T, bool)],
    output_map: &mut [OutputEntryN<T>],
) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    //println!("Start");
    let mut output_usages = vec![0; *input_len + clauses.len()];
    for (c, _) in clauses.iter() {
        for (l, _) in &c.literals {
            let l = usize::try_from(*l).unwrap();
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
    let oim_len = clauses.len() + *input_len;
    let mut oim = vec![0; oim_len];
    for (i, x) in output_map.iter().enumerate() {
        if let OutputEntryN::NewIndex(x, _) = x {
            oim[usize::try_from(*x).unwrap()] = i;
        }
    }

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

    let mut do_next_loop = false;
    //
    // traverse 1: resolve one literal clauses and resolve other clauses
    //
    for (o, _) in outputs.iter() {
        let o = usize::try_from(*o).unwrap();
        if o < *input_len {
            continue;
        }
        let o = match output_map[oim[o]] {
            OutputEntryN::NewIndex(o, _) => {
                let o = usize::try_from(o).unwrap();
                if o < *input_len {
                    continue;
                }
                o
            }
            OutputEntryN::Value(_) => continue,
        };
        let mut stack = Vec::<StackEntry>::new();
        stack.push(StackEntry {
            node: o - *input_len,
            way: 0,
            clause_id: None,
            negate_join: false,
        });
        while !stack.is_empty() {
            let mut top = stack.last_mut().unwrap();
            let node_index = top.node;
            let (clause, clause_neg) = &clauses[node_index];

            //println!("Stack top: {:?}", top);

            if top.way == 0 {
                if top.clause_id.is_some() {
                    // different visited masks for collection
                    if !visited_for_collect[node_index] {
                        visited_for_collect[node_index] = true;
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
                    // resolve negation
                    if top.negate_join {
                        target_clause.1 = !target_clause.1;
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
                // repeat process
                top.way = 0;
                top.clause_id = None;
            } else {
                let cur_out_n1 = if let OutputEntryN::NewIndex(_, n) =
                    output_map[oim[*input_len + node_index]]
                {
                    n
                } else {
                    panic!("Unexpected");
                };
                // resolve values and indexes for current clauses
                if clause.literals.is_empty() {
                    // fill up by zero ^ neg
                    output_map[oim[*input_len + node_index]] =
                        OutputEntryN::Value(*clause_neg ^ cur_out_n1);
                    do_next_loop = true;
                } else if clause.literals.len() == 1 {
                    // propagate to output_map
                    let l = usize::try_from(clause.literals[0].0).unwrap();
                    match output_map[oim[l]] {
                        OutputEntryN::NewIndex(x, n1) => {
                            output_map[oim[*input_len + node_index]] = OutputEntryN::NewIndex(
                                x,
                                cur_out_n1 ^ n1 ^ clause.literals[0].1 ^ *clause_neg,
                            );
                            // propagate usage of clause
                            output_usages[usize::try_from(x).unwrap()] +=
                                output_usages[*input_len + node_index] - 1;
                        }
                        OutputEntryN::Value(v) => {
                            output_map[oim[*input_len + node_index]] =
                                OutputEntryN::Value(v ^ clause.literals[0].1 ^ *clause_neg);
                        }
                    }
                    do_next_loop = true;
                } else {
                    // resolve clause
                    let mut new_literals = vec![];
                    let mut do_second_pass = false;
                    let mut neg_clause = false;
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
                            OutputEntryN::Value(v1) => {
                                let v = n ^ v1;
                                match clause.kind {
                                    ClauseKind::And => {
                                        if !v {
                                            new_literals.clear();
                                            break;
                                        }
                                    }
                                    ClauseKind::Xor => {
                                        if v {
                                            neg_clause = !neg_clause;
                                        }
                                    }
                                }
                                do_next_loop = true;
                            }
                        }
                    }
                    {
                        let (clause, clause_neg) = &mut clauses[node_index];
                        clause.literals = new_literals;
                        if neg_clause {
                            *clause_neg = !*clause_neg;
                        }
                        if clause.literals.len() >= 2 {
                            clause_len_before_second[node_index] = clause.literals.len();

                            if do_second_pass {
                                // prepare to second pass to collect clauses
                                //println!("Second pass: {:?}", clause);
                                top.way = 0; // reset way
                                top.clause_id = Some(node_index);
                                do_next_loop = true;
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
                                OutputEntryN::Value(v) => {
                                    output_map[oim[*input_len + node_index]] =
                                        OutputEntryN::Value(v ^ clause.literals[0].1 ^ *clause_neg);
                                }
                            }
                            do_next_loop = true;
                        } else {
                            // resolve empty clause
                            output_map[oim[*input_len + node_index]] =
                                OutputEntryN::Value(*clause_neg ^ cur_out_n1);
                            do_next_loop = true;
                        }
                    }
                }
                stack.pop();
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
        let o = match output_map[oim[o]] {
            OutputEntryN::NewIndex(o, _) => {
                let o = usize::try_from(o).unwrap();
                if o < *input_len {
                    continue;
                }
                o
            }
            OutputEntryN::Value(_) => continue,
        };
        let mut stack = Vec::<StackEntry>::new();
        stack.push(StackEntry {
            node: o - *input_len,
            way: 0,
            clause_id: None,
            negate_join: false,
        });
        while !stack.is_empty() {
            let mut top = stack.last_mut().unwrap();
            let node_index = top.node;
            let (clause, _) = &clauses[node_index];
            if top.way == 0 {
                if !used_new_outputs[*input_len + node_index] {
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
                    used_new_outputs[l] = true;
                }
            } else {
                stack.pop();
            }
        }
    }

    // include new usage from outputs
    for (o, _) in outputs.iter() {
        if let OutputEntryN::NewIndex(o, _) = output_map[usize::try_from(*o).unwrap()] {
            used_new_outputs[usize::try_from(o).unwrap()] = true;
        }
    }

    // translate literals map - from previous to current index
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
    for (clause, _) in clauses.iter_mut() {
        for (l, n) in &mut clause.literals {
            let l_u = usize::try_from(*l).unwrap();
            if let OutputEntryN::NewIndex(l1, n1) = output_map[oim[l_u]] {
                *l = trans_map[usize::try_from(l1).unwrap()];
                *n ^= n1;
            }
        }
    }
    for oe in output_map {
        if let OutputEntryN::NewIndex(i, n) = oe {
            *oe = OutputEntryN::NewIndex(trans_map[usize::try_from(*i).unwrap()], *n);
        }
    }
    *clauses = used_new_outputs[*input_len..]
        .iter()
        .enumerate()
        .filter(|(_, x)| **x)
        .map(|(i, _)| clauses[i].clone())
        .collect::<Vec<_>>();
    *input_len = used_new_outputs[..*input_len]
        .iter()
        .enumerate()
        .filter(|(_, x)| **x)
        .map(|(i, _)| trans_map[i])
        .map(|x| usize::try_from(x).unwrap())
        .max()
        .map(|x| x + 1)
        .unwrap_or_default();
    do_next_loop
}

// return optimized circuit, mapping to new inputs, mapping to new outputs
pub fn optimize_clause_circuit<T>(
    circuit: ClauseCircuit<T>,
) -> (ClauseCircuit<T>, Vec<Option<T>>, Vec<OutputEntry<T>>)
where
    T: Clone + Copy + Ord + PartialEq + Eq + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut clauses = circuit
        .clauses()
        .iter()
        .map(|x| (x.clone(), false))
        .collect::<Vec<_>>();

    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut output_map = (0..input_len + clauses.len())
        .map(|x| OutputEntryN::NewIndex(T::try_from(x).unwrap(), false))
        .collect::<Vec<_>>();

    let mut first = true;
    let mut new_input_len = input_len;
    while !reduce_clauses(&mut clauses) || first {
        // join clauses and remove unnecessary clauses
        first = false;
        if !join_and_remove_clauses(
            &mut new_input_len,
            &mut clauses,
            circuit.outputs(),
            &mut output_map,
        ) {
            break;
        }
    }

    (
        ClauseCircuit::new(T::default(), vec![], vec![]).unwrap(),
        vec![],
        vec![],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_clauses() {
        let mut clauses = [
            (
                Clause::new_and([(3, false), (0, false), (1, true), (3, false)]),
                false,
            ),
            (
                Clause::new_and([(3, true), (0, false), (1, true), (3, false)]),
                true,
            ),
            (
                Clause::new_and([(3, true), (3, true), (0, false), (1, true), (3, false)]),
                false,
            ),
            (
                Clause::new_and([(3, true), (0, false), (1, true), (3, true)]),
                false,
            ),
            (
                Clause::new_xor([(4, false), (3, false), (1, true), (2, false)]),
                false,
            ),
            (
                Clause::new_xor([(4, false), (2, false), (1, true), (2, false)]),
                true,
            ),
            (
                Clause::new_xor([(4, false), (2, false), (1, true), (2, true)]),
                true,
            ),
        ];
        assert!(reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(0, false), (1, true), (3, false)]), false),
                (Clause::new_and([]), true),
                (Clause::new_and([]), false),
                (Clause::new_and([(0, false), (1, true), (3, true)]), false),
                (
                    Clause::new_xor([(1, false), (2, false), (3, false), (4, false)]),
                    true
                ),
                (Clause::new_xor([(1, false), (4, false)]), false),
                (Clause::new_xor([(1, false), (4, false)]), true),
            ],
            clauses
        );

        // no changes
        let mut clauses = [
            (
                Clause::new_and([(3, false), (0, false), (1, true), (3, false)]),
                false,
            ),
            (
                Clause::new_xor([(4, false), (2, false), (1, true), (2, true)]),
                true,
            ),
        ];
        assert!(!reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(0, false), (1, true), (3, false)]), false),
                (Clause::new_xor([(1, false), (4, false)]), true),
            ],
            clauses
        );

        let mut clauses = [
            (
                Clause::new_and([(3, false), (0, false), (1, true), (3, false)]),
                false,
            ),
            (Clause::new_xor([(4, false), (2, false), (2, true)]), true),
        ];
        assert!(reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(0, false), (1, true), (3, false)]), false),
                (Clause::new_xor([(4, false)]), false),
            ],
            clauses
        );
    }

    #[test]
    fn test_join_and_remove_clauses() {
        // testcase
        // trivial no changes
        let mut input_len = 3;
        let mut clauses = vec![(Clause::new_and([(0, false), (1, false), (2, false)]), false)];
        let outputs = [(3, false)];
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
            &mut output_map
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
            let mut output_map = [OutputEntryN::NewIndex(0, t1)];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!([OutputEntryN::Value(t ^ t1),], output_map);
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
                    false,
                ),
                (Clause::new_and([(0, false), (1, false), (2, false)]), t),
            ];
            let outputs = [(3, false)];
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
                &mut output_map
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::Value(false),
                    OutputEntryN::Value(t ^ t1),
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
                    true,
                ),
                (Clause::new_and([(0, false), (1, false), (2, false)]), false),
            ];
            let outputs = [(3, false)];
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
                &mut output_map
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
                    OutputEntryN::Value(true),
                    OutputEntryN::NewIndex(2, false),
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
                    t,
                ),
                (Clause::new_xor([(0, false), (1, false), (2, false)]), t2),
            ];
            let outputs = [(3, false)];
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
                &mut output_map
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
                    OutputEntryN::Value(t ^ t1),
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
            let mut output_map = [OutputEntryN::NewIndex(0, t2), OutputEntryN::NewIndex(1, t3)];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map
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
                    true,
                ),
                (Clause::new_and([(0, t0), (1, false)]), t1),
            ];
            let outputs = [(2, false)];
            let mut output_map = [
                OutputEntryN::NewIndex(0, t2),
                OutputEntryN::NewIndex(1, false),
                OutputEntryN::NewIndex(2, t3),
            ];
            assert!(join_and_remove_clauses(
                &mut input_len,
                &mut clauses,
                &outputs,
                &mut output_map
            ));
            assert_eq!(1, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, t2),
                    OutputEntryN::Value(true),
                    OutputEntryN::NewIndex(0, t0 ^ t1 ^ t2 ^ t3),
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
                &mut output_map
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
                &mut output_map
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
                &mut output_map
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
                &mut output_map
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
                (Clause::new_and([]), false),
                (Clause::new_and([(2, false), (3, t), (4, false)]), false),
            ];
            let outputs = [(5, false)];
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
                &mut output_map
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses, "{}", tv);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::Value(false),
                    OutputEntryN::Value(false),
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
                &mut output_map
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
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3), // ???
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
                &mut output_map
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
                    OutputEntryN::NewIndex(3, t1 ^ t2 ^ t3), // ???
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
                &mut output_map
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
                &mut output_map
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
                &mut output_map
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
                (Clause::new_and([]), false),
                (Clause::new_and([(2, false), (4, t), (5, false)]), false),
            ];
            let outputs = [(6, false)];
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
                &mut output_map
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses, "{}", tv);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3), // ???
                    OutputEntryN::Value(false),
                    OutputEntryN::Value(false),
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
                (Clause::new_and([]), false),
                (Clause::new_xor([(3, t2), (4, false)]), t3),
                (Clause::new_and([]), false),
                (Clause::new_and([(2, false), (5, t), (6, false)]), false),
            ];
            let outputs = [(7, false)];
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
                &mut output_map
            ));
            assert_eq!(0, input_len);
            assert_eq!(Vec::<(Clause<usize>, bool)>::new(), clauses, "{}", tv);
            assert_eq!(
                [
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, false),
                    OutputEntryN::NewIndex(0, t1),
                    OutputEntryN::Value(false),
                    OutputEntryN::NewIndex(0, t1 ^ t2 ^ t3), // ???
                    OutputEntryN::Value(false),
                    OutputEntryN::Value(false),
                ],
                output_map,
                "{}",
                tv
            );
        }
    }
}
