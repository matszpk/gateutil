use gatesim::*;

use std::cmp::Ord;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter;

mod join_clauses;
use join_clauses::*;

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
                    clause.kind = ClauseKind::Xor; // empty Xor is false
                }
            }
            ClauseKind::Xor => {
                let mut pl = None;
                let mut new_literals = vec![];

                for (l, s) in &clause.literals {
                    *cs ^= s;
                    if let Some(xpl) = pl {
                        if xpl == l {
                            // we have l and l -> remove literal
                            new_literals.pop();
                            pl = None; // reset previous literal
                            continue;
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
        if old_len >= 1 && clause.len() < 2 {
            // return signal to next step if some clause have only 1 literal
            // or reduced to one or zero literals
            to_reduce_tree = true;
        }
    }
    to_reduce_tree
}

// return optimized circuit, mapping to new inputs, mapping to new outputs
pub fn optimize_clause_circuit<T>(
    circuit: ClauseCircuit<T>,
) -> (ClauseCircuit<T>, Vec<Option<T>>, Vec<OutputEntry<T>>)
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    //println!("OptStart");
    let mut clauses = circuit
        .clauses()
        .iter()
        .map(|x| (x.clone(), false))
        .collect::<Vec<_>>();

    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut output_map = (0..input_len + clauses.len())
        .map(|x| OutputEntryN::NewIndex(T::try_from(x).unwrap(), false))
        .collect::<Vec<_>>();

    let mut oim_opt = None;
    let mut new_input_len = input_len;
    loop {
        let mut do_next = reduce_clauses(&mut clauses);
        //println!("OptXPhase0: {:?}", clauses);
        // join clauses and remove unnecessary clauses
        do_next |= join_and_remove_clauses(
            &mut new_input_len,
            &mut clauses,
            circuit.outputs(),
            &mut output_map,
            &mut oim_opt,
        );
        //println!("OptXPhase: {:?}", clauses);
        //println!("OptXPhaseMap: {:?}", output_map);
        if !do_next {
            reduce_clauses(&mut clauses);
            //println!("OptXPhaseF: {:?}", clauses);
            break;
        }
    }

    // generate new clauses
    let mut new_clauses = clauses
        .iter()
        .map(|(clause, _)| clause.clone())
        .collect::<Vec<_>>();
    for clause in &mut new_clauses {
        for (l, n) in &mut clause.literals {
            // resolve sign of literal
            let l = usize::try_from(*l).unwrap();
            if l >= new_input_len {
                *n ^= clauses[l - new_input_len].1;
            }
        }
    }

    // new inputs
    let new_inputs = output_map[0..input_len]
        .iter()
        .map(|om| {
            if let OutputEntryN::NewIndex(x, _) = om {
                Some(*x)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // new outputs and new outputs map
    let mut new_outputs = vec![];
    let mut new_outputs_map = vec![OutputEntry::Value(false); circuit.outputs().len()];
    for (i, (o, on)) in circuit.outputs().iter().enumerate() {
        match output_map[usize::try_from(*o).unwrap()] {
            OutputEntryN::NewIndex(x, n) => {
                let no_idx = T::try_from(new_outputs.len()).unwrap();
                new_outputs_map[i] = OutputEntry::NewIndex(no_idx);
                let x_u = usize::try_from(x).unwrap();
                if x_u >= new_input_len {
                    new_outputs.push((x, on ^ n ^ clauses[x_u - new_input_len].1));
                } else {
                    new_outputs.push((x, on ^ n));
                }
            }
            OutputEntryN::Value(v) => {
                new_outputs_map[i] = OutputEntry::Value(v ^ on);
            }
        }
    }

    (
        ClauseCircuit::new(
            T::try_from(new_input_len).unwrap(),
            new_clauses,
            new_outputs,
        )
        .unwrap(),
        new_inputs,
        new_outputs_map,
    )
}

fn deduplicate_clauses<T>(clauses: &mut Vec<(usize, Option<usize>, Clause<T>)>) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let old_clause_len = clauses.len();
    clauses.sort_by_key(|(i, _, c)| (c.kind, c.literals.clone(), *i));
    let mut trans_table = HashMap::<usize, usize>::new();
    {
        let mut prev: Option<(usize, usize)> = None;
        for (i, (orig_i, _, clause)) in clauses.iter().enumerate() {
            if let Some((prev_i, prev_orig_i)) = prev {
                if clauses[prev_i].2 == *clause {
                    trans_table.insert(*orig_i, prev_orig_i);
                    continue;
                }
            }
            prev = Some((i, *orig_i));
        }
    }
    clauses.dedup_by_key(|(_, _, c)| (c.kind, c.literals.clone()));
    let new_clause_len = clauses.len();
    // translate literals and sort and deduplicate literals
    for (_, _, clause) in clauses {
        for (l, _) in &mut clause.literals {
            let l_u = usize::try_from(*l).unwrap();
            if let Some(trans_l) = trans_table.get(&l_u) {
                *l = T::try_from(*trans_l).unwrap();
            }
        }
        clause.literals.sort();
        if clause.kind == ClauseKind::And {
            clause.literals.dedup();
        }
    }
    old_clause_len != new_clause_len
}

// remove b from a
pub fn remove_sorted_ref<'a, T, I2>(a: &mut Vec<T>, b: I2)
where
    T: Clone + Copy + Default + std::cmp::Ord + 'a,
    I2: IntoIterator<Item = &'a T>,
{
    let mut b = b.into_iter();
    let alen = a.len();
    let mut i = 0;
    let mut j = 0;
    while let Some(bv) = b.next() {
        while i < a.len() && a[i] < *bv {
            a[j] = a[i];
            i += 1;
            j += 1;
        }
        if i == alen {
            break;
        }
        if a[i] == *bv {
            i += 1;
        }
    }
    while i < alen {
        a[j] = a[i];
        i += 1;
        j += 1;
    }
    a.resize(j, T::default());
}

pub fn sorted_is_set_contains_set<T: Copy + std::cmp::Ord>(a: &[T], b: &[T]) -> bool {
    let (mut ai, mut bi) = (0, 0);
    while ai < a.len() {
        let (ac, bc) = (a[ai], b[bi]);
        if ac < bc {
            break;
        } else if ac > bc {
            bi += 1;
            if bi >= b.len() {
                break;
            }
        } else {
            ai += 1;
        }
    }
    ai == a.len()
}

// TreeNode for traversing between tree structure for clause-chain (clause-tree).

struct TreeNode<T> {
    value: T,
    children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            children: None,
        }
    }

    fn append_child(&mut self, child: TreeNode<T>) {
        if let Some(children) = &mut self.children {
            children.push(child);
        } else {
            self.children = Some(vec![child]);
        }
    }

    fn stack_iter<'a>(&'a self) -> TreeStackIterator<'a, T> {
        TreeStackIterator::new(self)
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        TreeStackIterator::new(self).filter_map(|(op, x)| {
            if op == TreeStackOp::Push {
                Some(x)
            } else {
                None
            }
        })
    }
}

struct TreeStackElem<'a, T> {
    node: &'a TreeNode<T>,
    child_index: Option<usize>,
}

struct TreeStackIterator<'a, T>(Vec<TreeStackElem<'a, T>>);

impl<'a, T> TreeStackIterator<'a, T> {
    fn new(root: &'a TreeNode<T>) -> Self {
        Self(vec![TreeStackElem {
            node: root,
            child_index: None,
        }])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeStackOp {
    Push,
    Pop,
}

impl<'a, T> Iterator for TreeStackIterator<'a, T> {
    type Item = (TreeStackOp, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(top) = self.0.last_mut() {
            if let Some(child_index) = top.child_index {
                if top
                    .node
                    .children
                    .as_ref()
                    .map(|ch| child_index < ch.len())
                    .unwrap_or_default()
                {
                    top.child_index = Some(child_index + 1);
                    let child = &top.node.children.as_ref().unwrap()[child_index];
                    let value = &child.value;
                    self.0.push(TreeStackElem {
                        node: child,
                        child_index: Some(0),
                    });
                    Some((TreeStackOp::Push, value))
                } else {
                    let value = &top.node.value;
                    self.0.pop();
                    Some((TreeStackOp::Pop, &value))
                }
            } else {
                let value = &top.node.value;
                if let Some(children) = &top.node.children {
                    if children.is_empty() {
                        self.0.pop();
                        Some((TreeStackOp::Pop, &value))
                    } else {
                        top.child_index = Some(0);
                        Some((TreeStackOp::Push, value))
                    }
                } else {
                    self.0.pop();
                    Some((TreeStackOp::Pop, &value))
                }
            }
        } else {
            None
        }
    }
}

// return extra clauses with range of placement.
// argument is clause slice: element: (clause_index, Option<extra_clause_index>, clause)
// extra_clause_index >= input_len + total_clause_num
// if extra_clause_index is not None - new index of new extra clause
// if extra_clause_index is None - old clause
// if clause empty and extra_clause_index is not None -
//    clause_index - original index of removed clause
//    extra_clause_index - index of clause that replace removed clause.
// extra_clause_start - start index for new extra clauses
fn deduplicate_literal_clauses<T>(
    input_len: usize,
    total_clause_num: usize,
    extra_clause_start: usize,
    clauses: &mut Vec<(usize, Option<usize>, Clause<T>)>,
) where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    if clauses.is_empty() {
        return;
    }
    let kind = clauses.first().unwrap().2.kind;

    let clause_num = clauses.len();
    let total_output_num = input_len + total_clause_num;
    let same_occur_lits = {
        let mut lit_clause_tbl = vec![(0, vec![]); total_output_num << 1];
        for (i, (l, _)) in lit_clause_tbl.iter_mut().enumerate() {
            *l = i;
        }
        for (i, (_, _, clause)) in clauses.iter().enumerate() {
            for (l, n) in &clause.literals {
                let l = (usize::try_from(*l).unwrap() << 1) + usize::from(*n);
                lit_clause_tbl[l].1.push(i);
            }
        }
        for (_, occurs) in &mut lit_clause_tbl {
            occurs.sort();
        }
        lit_clause_tbl.sort_by_key(|(_, o)| o.clone());
        let mut prev: Option<Vec<usize>> = None;
        // collect literals with same occurrence into same list
        let mut same_occur_lits: Vec<(Vec<(T, bool)>, Vec<usize>)> = vec![];
        for (l, occurs) in lit_clause_tbl.drain(..) {
            if let Some(p) = prev {
                if p.len() >= 2 && p == occurs {
                    same_occur_lits
                        .last_mut()
                        .unwrap()
                        .0
                        .push((T::try_from(l >> 1).unwrap(), (l & 1) != 0));
                    prev = Some(occurs);
                    continue;
                }
            }
            same_occur_lits.push((
                vec![(T::try_from(l >> 1).unwrap(), (l & 1) != 0)],
                occurs.clone(),
            ));
            prev = Some(occurs);
        }
        // sort before using
        for (same_lits, _) in &mut same_occur_lits {
            same_lits.sort();
        }
        same_occur_lits
    };

    let mut j = 0;
    // apply same occurrence literals list (clauses) into clauses
    for (same_lits, occurs) in same_occur_lits.into_iter() {
        if same_lits.len() > 1 {
            for occur in &occurs {
                let clause = &mut clauses[*occur].2;
                remove_sorted_ref(&mut clause.literals, &same_lits);
                let extra_lit = T::try_from(extra_clause_start + j).unwrap();
                clause.literals.push((extra_lit, false));
            }
            clauses.push((
                *occurs.first().unwrap(),
                Some(extra_clause_start + j),
                Clause {
                    kind,
                    literals: same_lits.clone(),
                },
            ));
            j += 1;
        }
    }
    // sort clause literals
    for (_, _, clause) in clauses.iter_mut() {
        clause.literals.sort();
    }

    // algorithm: first find smallest subclauses with greatest occurrences.

    loop {
        // get pair_count_map sorted by count descending
        let mut pairlit_clause_map = {
            let mut pairlit_clause_map = HashMap::<((T, bool), (T, bool)), Vec<usize>>::new();
            for (ci, (_, _, clause)) in clauses.iter().enumerate() {
                for (i, ls1) in clause.literals.iter().enumerate() {
                    for ls2 in &clause.literals[i + 1..] {
                        if let Some(list) = pairlit_clause_map.get_mut(&(*ls1, *ls2)) {
                            list.push(ci);
                        } else {
                            pairlit_clause_map.insert((*ls1, *ls2), vec![ci]);
                        }
                    }
                }
            }
            let mut pairlit_clause_map = Vec::from_iter(pairlit_clause_map.into_iter());
            pairlit_clause_map.sort_by_key(|(k, list)| (std::cmp::Reverse(list.len()), *k));
            pairlit_clause_map
        };

        let mut chain_found = false;
        let threshold = std::cmp::max((pairlit_clause_map.len() + 9) / 10, 9);
        for ((ls1, ls2), list) in &mut pairlit_clause_map[0..threshold] {
            list.sort_by_key(|ci| (clauses[*ci].2.len(), clauses[*ci].2.literals.clone()));
            // find clause chain
            let mut prev = Option::<usize>::None;
            for ci in list {
                if let Some(prev_ci) = prev {
                    if sorted_is_set_contains_set(
                        &clauses[prev_ci].2.literals,
                        &clauses[*ci].2.literals,
                    ) {}
                }
                prev = Some(*ci);
            }
        }

        if !chain_found {
            break;
        }
    }

    // final clauses
    clauses.sort_by_key(|(orig_idx, extra_idx, _)| (*orig_idx, *extra_idx));
}

pub fn merge_sorted_by_key<T, I1, I2, F, B>(a: I1, b: I2, mut f: F) -> Vec<T>
where
    T: Clone,
    I1: IntoIterator<Item = T>,
    I2: IntoIterator<Item = T>,
    F: FnMut(&T) -> B,
    B: std::cmp::Ord,
{
    let mut sorted = vec![];
    let (mut ai, mut bi) = (a.into_iter(), b.into_iter());
    let (mut av, mut bv) = (ai.next().clone(), bi.next().clone());
    if av.is_none() {
        if let Some(bv) = bv {
            sorted.push(bv);
            sorted.extend(bi);
        }
    } else if bv.is_none() {
        if let Some(av) = av {
            sorted.push(av);
            sorted.extend(ai);
        }
    } else {
        loop {
            if f(av.as_ref().unwrap()) < f(bv.as_ref().unwrap()) {
                sorted.push(av.take().unwrap());
                av = ai.next();
                if av.is_none() {
                    sorted.push(bv.take().unwrap());
                    sorted.extend(bi);
                    break;
                }
            } else {
                sorted.push(bv.take().unwrap());
                bv = bi.next();
                if bv.is_none() {
                    sorted.push(av.take().unwrap());
                    sorted.extend(ai);
                    break;
                }
            }
        }
    }
    sorted
}

fn join_deduplicates_to_clause_circuit<T>(
    input_len: usize,
    total_clause_num: usize,
    and_clauses: Vec<(usize, Option<usize>, Clause<T>)>,
    xor_clauses: Vec<(usize, Option<usize>, Clause<T>)>,
    outputs: &[(T, bool)],
) -> ClauseCircuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut out_clauses =
        merge_sorted_by_key(and_clauses, xor_clauses, |(orig_idx, extra_idx, _)| {
            (*orig_idx, *extra_idx)
        });
    let mut trans_table = vec![0; input_len + total_clause_num];
    for (i, (j, extra_j, _)) in out_clauses.iter().enumerate() {
        if let Some(ej) = extra_j {
            trans_table[*ej] = i + input_len;
        } else {
            trans_table[*j] = i + input_len;
        }
    }
    for (_, _, clause) in &mut out_clauses {
        for (l, _) in &mut clause.literals {
            let l_u = usize::try_from(*l).unwrap();
            if l_u >= input_len {
                *l = T::try_from(trans_table[l_u]).unwrap();
            }
        }
    }
    ClauseCircuit::new(
        T::try_from(input_len).unwrap(),
        out_clauses
            .into_iter()
            .map(|(_, _, c)| c)
            .filter(|c| c.len() != 0),
        outputs.iter().map(|(l, n)| {
            let l_u = usize::try_from(*l).unwrap();
            if l_u >= input_len {
                (T::try_from(trans_table[l_u]).unwrap(), *n)
            } else {
                (*l, *n)
            }
        }),
    )
    .unwrap()
}

pub fn check_if_clauses_need_optimization_and_fix<T>(
    clauses: &mut [(usize, Option<usize>, Clause<T>)],
) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    for (_, _, clause) in clauses {
        if clause.literals.len() == 1 {
            clause.literals.push(*clause.literals.first().unwrap());
            clause.kind = ClauseKind::And;
            return true;
        }
        let mut prev = None;
        for (l, _) in &clause.literals {
            if let Some(prev_l) = prev {
                if prev_l == *l {
                    // and((l,false), (l,true)) -> false
                    // or xor((l,false), (l,true)) -> true
                    // or xor((l,false), (l,true)) -> false (duplicates allowed)
                    return true;
                }
            }
            prev = Some(*l);
        }
    }
    false
}

// deduplicate clauses and clause literals
// return new circuit and boolean value.
// if some possible literal duplicates then returns true, otherwise return false
pub fn deduplicate_clause_circuit<T>(circuit: ClauseCircuit<T>) -> (ClauseCircuit<T>, bool)
where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    // assertion for sorted and deduplicated clauses
    assert!(circuit.clauses().iter().all(|c| {
        let mut prev = None;
        for l in &c.literals {
            if let Some(p) = prev {
                if !(p < l) {
                    return false;
                }
            }
            prev = Some(l);
        }
        true
    }));
    let input_len = usize::try_from(circuit.input_len()).unwrap();
    // return (clause_index, Option<extra_clause_index>, clause) vector
    let mut and_clauses = circuit
        .clauses()
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.kind == ClauseKind::And {
                Some((input_len + i, Option::<usize>::None, c.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let and_clauses_need_optim = if deduplicate_clauses(&mut and_clauses) {
        // check whether clauses need optimizations
        check_if_clauses_need_optimization_and_fix(&mut and_clauses)
    } else {
        false
    };

    // return (clause_index, Option<extra_clause_index>, clause) vector
    let mut xor_clauses = circuit
        .clauses()
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.kind == ClauseKind::Xor {
                Some((input_len + i, Option::<usize>::None, c.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let xor_clauses_need_optim = if deduplicate_clauses(&mut xor_clauses) {
        check_if_clauses_need_optimization_and_fix(&mut xor_clauses)
    } else {
        false
    };

    let old_and_clauses_len = and_clauses.len();
    if !and_clauses_need_optim {
        deduplicate_literal_clauses(input_len, circuit.len(), circuit.len(), &mut and_clauses);
    }

    let old_xor_clauses_len = xor_clauses.len();
    if !xor_clauses_need_optim {
        deduplicate_literal_clauses(
            input_len,
            circuit.len(),
            circuit.len() + and_clauses.len() - old_and_clauses_len,
            &mut xor_clauses,
        );
    }

    (
        join_deduplicates_to_clause_circuit(
            input_len,
            circuit.len()
                + (and_clauses.len() - old_and_clauses_len)
                + (xor_clauses.len() - old_xor_clauses_len),
            and_clauses,
            xor_clauses,
            circuit.outputs(),
        ),
        and_clauses_need_optim | xor_clauses_need_optim,
    )
}

pub fn assign_to_circuit_and_optimize<T>(
    circuit: &Circuit<T>,
    inputs: impl IntoIterator<Item = (T, bool)>,
    seq: bool,
) -> (Circuit<T>, Vec<OutputEntry<T>>, Vec<OutputEntry<T>>)
where
    T: Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Debug,
    T: TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let (circuit, input_map, output_map) = assign_to_circuit(circuit, inputs);
    let clause_circuit = ClauseCircuit::from(circuit);
    //println!("ClauseCircuit: {:?}", clause_circuit);
    let (opt_circuit, opt_input_map, opt_output_map) = optimize_clause_circuit(clause_circuit);
    let opt_circuit = if seq {
        Circuit::from_seq(opt_circuit)
    } else {
        Circuit::from(opt_circuit)
    };
    let mut out_input_map = vec![OutputEntry::Value(false); input_map.len()];
    for (i, e) in input_map.into_iter().enumerate() {
        out_input_map[i] = match e {
            OutputEntry::NewIndex(x) => {
                let x = usize::try_from(x).unwrap();
                match opt_input_map[x] {
                    Some(x) => OutputEntry::NewIndex(x),
                    None => OutputEntry::Value(false),
                }
            }
            OutputEntry::Value(v) => OutputEntry::Value(v),
        };
    }
    let mut out_output_map = vec![OutputEntry::Value(false); output_map.len()];
    for (i, e) in output_map.into_iter().enumerate() {
        out_output_map[i] = match e {
            OutputEntry::NewIndex(x) => {
                let x = usize::try_from(x).unwrap();
                match opt_output_map[x] {
                    OutputEntry::NewIndex(x) => OutputEntry::NewIndex(x),
                    OutputEntry::Value(v) => OutputEntry::Value(v),
                }
            }
            OutputEntry::Value(v) => OutputEntry::Value(v),
        };
    }
    (opt_circuit, out_input_map, out_output_map)
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
                (Clause::new_xor([]), true),
                (Clause::new_xor([]), false),
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

        let mut clauses = [
            (Clause::new_and([(3, false)]), false),
            (Clause::new_xor([(4, false), (2, false), (3, true)]), true),
        ];
        assert!(reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(3, false)]), false),
                (Clause::new_xor([(2, false), (3, false), (4, false)]), false),
            ],
            clauses
        );

        for i in 1..8 {
            let mut clauses = [(
                Clause::new_and(std::iter::repeat((2, false)).take(i)),
                false,
            )];
            assert!(reduce_clauses(&mut clauses));
            assert_eq!([(Clause::new_and([(2, false)]), false),], clauses);
        }

        for i in 1..8 {
            let mut clauses = [(
                Clause::new_xor(std::iter::repeat((2, false)).take(i)),
                false,
            )];
            assert!(reduce_clauses(&mut clauses));
            assert_eq!(
                [(
                    if (i & 1) != 0 {
                        Clause::new_xor([(2, false)])
                    } else {
                        Clause::new_xor([])
                    },
                    false
                )],
                clauses
            );
        }
    }

    #[test]
    fn test_deduplicate_clauses() {
        let mut clauses = vec![
            (
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false)]),
            ),
            (4, None, Clause::new_and([(0, false), (1, true)])),
            (5, None, Clause::new_and([(0, false), (2, true)])),
            (
                8,
                None,
                Clause::new_and([(3, true), (4, false), (6, false)]),
            ),
            (6, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(deduplicate_clauses(&mut clauses));
        assert_eq!(
            vec![
                (4, None, Clause::new_and([(0, false), (1, true)])),
                (5, None, Clause::new_and([(0, false), (2, true)])),
                (
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false)])
                ),
                (
                    8,
                    None,
                    Clause::new_and([(3, true), (4, false), (5, false)])
                ),
            ],
            clauses
        );

        let mut clauses = vec![
            (
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false)]),
            ),
            (5, None, Clause::new_and([(0, false), (1, true)])),
            (4, None, Clause::new_and([(0, false), (2, true)])),
            (
                8,
                None,
                Clause::new_and([(3, true), (5, false), (6, false)]),
            ),
            (6, None, Clause::new_and([(0, false), (2, true)])),
            (9, None, Clause::new_and([(0, false), (2, true)])),
            (
                10,
                None,
                Clause::new_and([(1, true), (2, false), (9, false)]),
            ),
        ];
        assert!(deduplicate_clauses(&mut clauses));
        assert_eq!(
            vec![
                (5, None, Clause::new_and([(0, false), (1, true)])),
                (4, None, Clause::new_and([(0, false), (2, true)])),
                (
                    10,
                    None,
                    Clause::new_and([(1, true), (2, false), (4, false)]),
                ),
                (
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false)])
                ),
                (
                    8,
                    None,
                    Clause::new_and([(3, true), (4, false), (5, false)])
                ),
            ],
            clauses
        );

        let mut clauses = vec![
            (
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false)]),
            ),
            (4, None, Clause::new_and([(0, false), (1, true)])),
            (5, None, Clause::new_and([(0, false), (2, true)])),
            (
                8,
                None,
                Clause::new_and([(3, true), (4, false), (6, false)]),
            ),
            (6, None, Clause::new_xor([(0, false), (2, true)])),
        ];
        assert!(!deduplicate_clauses(&mut clauses));
        assert_eq!(
            vec![
                (4, None, Clause::new_and([(0, false), (1, true)])),
                (5, None, Clause::new_and([(0, false), (2, true)])),
                (
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false)])
                ),
                (
                    8,
                    None,
                    Clause::new_and([(3, true), (4, false), (6, false)])
                ),
                (6, None, Clause::new_xor([(0, false), (2, true)]))
            ],
            clauses
        );

        // link two duplicates to some clause. and remove one.
        let mut clauses = vec![
            (
                7,
                None,
                Clause::new_and([(1, true), (3, false), (4, false)]),
            ),
            (4, None, Clause::new_and([(0, false), (1, true)])),
            (5, None, Clause::new_and([(0, false), (2, true)])),
            (
                8,
                None,
                Clause::new_and([(3, true), (5, false), (6, false)]),
            ),
            (6, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(deduplicate_clauses(&mut clauses));
        assert_eq!(
            vec![
                (4, None, Clause::new_and([(0, false), (1, true)])),
                (5, None, Clause::new_and([(0, false), (2, true)])),
                (
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (4, false)])
                ),
                (8, None, Clause::new_and([(3, true), (5, false)]))
            ],
            clauses
        );

        // link two duplicates to some clause. and do not remove one because is xor.
        let mut clauses = vec![
            (
                7,
                None,
                Clause::new_xor([(1, true), (3, false), (4, false)]),
            ),
            (4, None, Clause::new_xor([(0, false), (1, true)])),
            (5, None, Clause::new_xor([(0, false), (2, true)])),
            (
                8,
                None,
                Clause::new_xor([(3, true), (5, false), (6, false)]),
            ),
            (6, None, Clause::new_xor([(0, false), (2, true)])),
        ];
        assert!(deduplicate_clauses(&mut clauses));
        assert_eq!(
            vec![
                (4, None, Clause::new_xor([(0, false), (1, true)])),
                (5, None, Clause::new_xor([(0, false), (2, true)])),
                (
                    7,
                    None,
                    Clause::new_xor([(1, true), (3, false), (4, false)])
                ),
                (
                    8,
                    None,
                    Clause::new_xor([(3, true), (5, false), (5, false)])
                )
            ],
            clauses
        );

        // link two duplicates to some clause.
        // and do not remove any because negation is different.
        let mut clauses = vec![
            (
                7,
                None,
                Clause::new_and([(1, true), (3, false), (4, false)]),
            ),
            (4, None, Clause::new_and([(0, false), (1, true)])),
            (5, None, Clause::new_and([(0, false), (2, true)])),
            (8, None, Clause::new_and([(3, true), (5, false), (6, true)])),
            (6, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(deduplicate_clauses(&mut clauses));
        assert_eq!(
            vec![
                (4, None, Clause::new_and([(0, false), (1, true)])),
                (5, None, Clause::new_and([(0, false), (2, true)])),
                (
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (4, false)])
                ),
                (8, None, Clause::new_and([(3, true), (5, false), (5, true)]))
            ],
            clauses
        );
    }

    #[test]
    fn test_merge_sorted_by_key() {
        assert_eq!(
            vec![1, 2, 4, -4, 6, 6, 8, 9, -11],
            merge_sorted_by_key(vec![1, 2, -4, 6, 9], vec![4, 6, 8, -11], |l: &isize| l
                .abs())
        );
        assert_eq!(
            vec![1, 2, -4, 6, 9],
            merge_sorted_by_key(vec![1, 2, -4, 6, 9], vec![], |l: &isize| l.abs())
        );
        assert_eq!(
            vec![1, 3, 5, -6, -7, 8, 11, 12, 16],
            merge_sorted_by_key(vec![5, -7, 11, 12, 16], vec![1, 3, -6, 8], |l: &isize| l
                .abs())
        );
        assert_eq!(
            vec![1, 3, -6, 8],
            merge_sorted_by_key(vec![], vec![1, 3, -6, 8], |l: &isize| l.abs())
        );
    }

    #[test]
    fn test_join_deduplicates_to_clause_circuit() {
        assert_eq!(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, true)]),
                    Clause::new_and([(0, false), (3, true)]),
                    Clause::new_and([(1, true), (2, true), (4, false), (5, false)]),
                    Clause::new_xor([(0, false), (3, true)]),
                    Clause::new_xor([(0, false), (2, true)]),
                    Clause::new_xor([(1, true), (3, true), (7, false), (8, false)]),
                ],
                [(6, false), (9, false)]
            )
            .unwrap(),
            join_deduplicates_to_clause_circuit(
                4,
                6,
                vec![
                    (4, None, Clause::new_and([(0, false), (1, true)])),
                    (4, Some(8), Clause::new_and([(0, false), (3, true)])),
                    (
                        5,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (8, false)])
                    ),
                ],
                vec![
                    (6, None, Clause::new_xor([(0, false), (3, true)])),
                    (6, Some(9), Clause::new_xor([(0, false), (2, true)])),
                    (
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (6, false), (9, false)])
                    ),
                ],
                &[(5, false), (7, false)]
            )
        );

        assert_eq!(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, true)]),
                    Clause::new_and([(0, false), (3, true)]),
                    Clause::new_xor([(0, false), (3, true)]),
                    Clause::new_xor([(0, false), (2, true)]),
                    Clause::new_and([(1, true), (2, true), (4, false), (5, false)]),
                    Clause::new_xor([(1, true), (3, true), (6, false), (7, false)]),
                ],
                [(8, false), (9, false)]
            )
            .unwrap(),
            join_deduplicates_to_clause_circuit(
                4,
                6,
                vec![
                    (4, None, Clause::new_and([(0, false), (1, true)])),
                    (4, Some(8), Clause::new_and([(0, false), (3, true)])),
                    (
                        6,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (8, false)])
                    ),
                ],
                vec![
                    (5, None, Clause::new_xor([(0, false), (3, true)])),
                    (5, Some(9), Clause::new_xor([(0, false), (2, true)])),
                    (
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (5, false), (9, false)])
                    ),
                ],
                &[(6, false), (7, false)]
            )
        );

        assert_eq!(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, true)]),
                    Clause::new_and([(0, false), (3, true), (4, false)]),
                    Clause::new_xor([(0, false), (3, true)]),
                    Clause::new_xor([(0, false), (2, true), (6, true)]),
                    Clause::new_and([(1, true), (2, true), (4, false), (5, false)]),
                    Clause::new_xor([(1, true), (3, true), (6, false), (7, false)]),
                ],
                [(8, false), (9, false)]
            )
            .unwrap(),
            join_deduplicates_to_clause_circuit(
                4,
                6,
                vec![
                    (4, None, Clause::new_and([(0, false), (1, true)])),
                    (
                        4,
                        Some(8),
                        Clause::new_and([(0, false), (3, true), (4, false)])
                    ),
                    (
                        6,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (8, false)])
                    ),
                ],
                vec![
                    (5, None, Clause::new_xor([(0, false), (3, true)])),
                    (
                        5,
                        Some(9),
                        Clause::new_xor([(0, false), (2, true), (5, true)])
                    ),
                    (
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (5, false), (9, false)])
                    ),
                ],
                &[(6, false), (7, false)]
            )
        );
        assert_eq!(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, true)]),
                    Clause::new_and([(0, false), (3, true), (4, false)]),
                    Clause::new_and([(0, false), (5, false)]),
                    Clause::new_xor([(0, false), (3, true)]),
                    Clause::new_xor([(0, false), (2, true), (7, true)]),
                    Clause::new_xor([(2, true), (8, true)]),
                    Clause::new_and([(1, true), (2, true), (4, false), (6, false)]),
                    Clause::new_xor([(1, true), (3, true), (7, false), (9, false)]),
                ],
                [(10, false), (11, false)]
            )
            .unwrap(),
            join_deduplicates_to_clause_circuit(
                4,
                8,
                vec![
                    (4, None, Clause::new_and([(0, false), (1, true)])),
                    (
                        4,
                        Some(8),
                        Clause::new_and([(0, false), (3, true), (4, false)])
                    ),
                    (4, Some(10), Clause::new_and([(0, false), (8, false)])),
                    (
                        6,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (10, false)])
                    ),
                ],
                vec![
                    (5, None, Clause::new_xor([(0, false), (3, true)])),
                    (
                        5,
                        Some(9),
                        Clause::new_xor([(0, false), (2, true), (5, true)])
                    ),
                    (5, Some(11), Clause::new_xor([(2, true), (9, true)])),
                    (
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (5, false), (11, false)])
                    ),
                ],
                &[(6, false), (7, false)]
            )
        );
        assert_eq!(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, true)]),
                    Clause::new_xor([(0, false), (3, true)]),
                    Clause::new_and([(0, false), (2, true)]),
                    Clause::new_and([(0, false), (3, true), (4, false)]),
                    Clause::new_xor([(1, false), (3, true)]),
                    Clause::new_xor([(0, false), (2, true), (5, true)]),
                    Clause::new_and([(1, true), (2, true), (6, false), (7, false)]),
                    Clause::new_xor([(1, true), (3, true), (8, false), (9, false)])
                ],
                [(10, false), (11, false)]
            )
            .unwrap(),
            join_deduplicates_to_clause_circuit(
                4,
                8,
                vec![
                    (4, None, Clause::new_and([(0, false), (1, true)])),
                    (6, None, Clause::new_and([(0, false), (2, true)])),
                    (
                        6,
                        Some(10),
                        Clause::new_and([(0, false), (3, true), (4, false)])
                    ),
                    (
                        8,
                        None,
                        Clause::new_and([(1, true), (2, true), (6, false), (10, false)])
                    ),
                ],
                vec![
                    (5, None, Clause::new_xor([(0, false), (3, true)])),
                    (7, None, Clause::new_xor([(1, false), (3, true)])),
                    (
                        7,
                        Some(11),
                        Clause::new_xor([(0, false), (2, true), (5, true)])
                    ),
                    (
                        9,
                        None,
                        Clause::new_xor([(1, true), (3, true), (7, false), (11, false)])
                    ),
                ],
                &[(8, false), (9, false)]
            )
        );
    }

    #[test]
    fn test_check_if_clauses_need_optimization_and_fix() {
        let mut clauses = vec![
            (
                4,
                None,
                Clause::new_and([(0, false), (1, true), (2, false)]),
            ),
            (5, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(!check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                (
                    4,
                    None,
                    Clause::new_and([(0, false), (1, true), (2, false)])
                ),
                (5, None, Clause::new_and([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            (
                4,
                None,
                Clause::new_and([(0, false), (1, true), (1, false)]),
            ),
            (5, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                (
                    4,
                    None,
                    Clause::new_and([(0, false), (1, true), (1, false)])
                ),
                (5, None, Clause::new_and([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            (4, None, Clause::new_xor([(0, false), (1, true), (1, true)])),
            (5, None, Clause::new_xor([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                (4, None, Clause::new_xor([(0, false), (1, true), (1, true)])),
                (5, None, Clause::new_xor([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            (4, None, Clause::new_and([(0, true)])),
            (5, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                (4, None, Clause::new_and([(0, true), (0, true)])),
                (5, None, Clause::new_and([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            (4, None, Clause::new_xor([(0, true)])),
            (5, None, Clause::new_xor([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                (4, None, Clause::new_and([(0, true), (0, true)])),
                (5, None, Clause::new_xor([(0, false), (2, true)])),
            ]
        );
    }

    #[test]
    fn test_remove_sorted_ref() {
        let mut avec = vec![0, 3, 5, 7, 8];
        remove_sorted_ref(&mut avec, &[1, 4, 6]);
        assert_eq!(vec![0, 3, 5, 7, 8], avec);

        let mut avec = vec![0, 3, 4, 5, 7, 8];
        remove_sorted_ref(&mut avec, &[1, 4, 7]);
        assert_eq!(vec![0, 3, 5, 8], avec);

        let mut avec = vec![1, 3, 4, 5, 7, 8];
        remove_sorted_ref(&mut avec, &[1, 4, 8]);
        assert_eq!(vec![3, 5, 7], avec);

        let mut avec = vec![1, 3, 4, 5, 7, 8];
        remove_sorted_ref(&mut avec, &[]);
        assert_eq!(vec![1, 3, 4, 5, 7, 8], avec);

        let mut avec = vec![];
        remove_sorted_ref(&mut avec, &[5, 6, 11]);
        assert_eq!(Vec::<u32>::new(), avec);

        let mut avec = vec![1, 3, 4, 5, 7, 8];
        remove_sorted_ref(&mut avec, &[0, 9]);
        assert_eq!(vec![1, 3, 4, 5, 7, 8], avec);

        let mut avec = vec![1, 3, 4, 5, 7, 8];
        remove_sorted_ref(&mut avec, &[0, 5, 9]);
        assert_eq!(vec![1, 3, 4, 7, 8], avec);

        let mut avec = vec![1, 3, 4, 5, 7, 8];
        remove_sorted_ref(&mut avec, &[0]);
        assert_eq!(vec![1, 3, 4, 5, 7, 8], avec);
    }

    #[test]
    fn test_tree() {
        use TreeStackOp::*;
        let root = TreeNode {
            value: 1,
            children: Some(vec![
                TreeNode {
                    value: 2,
                    children: None,
                },
                TreeNode {
                    value: 4,
                    children: Some(vec![
                        TreeNode {
                            value: 5,
                            children: None,
                        },
                        TreeNode {
                            value: 6,
                            children: None,
                        },
                        TreeNode {
                            value: 7,
                            children: Some(vec![
                                TreeNode {
                                    value: 11,
                                    children: None,
                                },
                                TreeNode {
                                    value: 13,
                                    children: None,
                                },
                            ]),
                        },
                    ]),
                },
                TreeNode {
                    value: 3,
                    children: Some(vec![
                        TreeNode {
                            value: 8,
                            children: None,
                        },
                        TreeNode {
                            value: 9,
                            children: Some(vec![
                                TreeNode {
                                    value: 12,
                                    children: None,
                                },
                                TreeNode {
                                    value: 14,
                                    children: None,
                                },
                            ]),
                        },
                        TreeNode {
                            value: 10,
                            children: None,
                        },
                    ]),
                },
            ]),
        };
        assert_eq!(
            vec![
                (Push, 1),
                (Push, 2),
                (Pop, 2),
                (Push, 4),
                (Push, 5),
                (Pop, 5),
                (Push, 6),
                (Pop, 6),
                (Push, 7),
                (Push, 11),
                (Pop, 11),
                (Push, 13),
                (Pop, 13),
                (Pop, 7),
                (Pop, 4),
                (Push, 3),
                (Push, 8),
                (Pop, 8),
                (Push, 9),
                (Push, 12),
                (Pop, 12),
                (Push, 14),
                (Pop, 14),
                (Pop, 9),
                (Push, 10),
                (Pop, 10),
                (Pop, 3),
                (Pop, 1)
            ],
            Vec::from_iter(root.stack_iter().map(|(op, x)| (op, *x)))
        );

        assert_eq!(
            vec![1, 2, 4, 5, 6, 7, 11, 13, 3, 8, 9, 12, 14, 10],
            Vec::from_iter(root.iter().copied())
        );
    }
}
