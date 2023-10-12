use gatesim::*;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct DedupClause<T> {
    pub(crate) orig_index: T,
    pub(crate) extra_index: Option<T>,
    pub(crate) clause: Clause<T>,
}

impl<T: Ord> PartialOrd for DedupClause<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.orig_index == other.orig_index {
            Some(self.extra_index.as_ref().cmp(&other.extra_index.as_ref()))
        } else {
            Some(self.orig_index.cmp(&other.orig_index))
        }
    }
}

impl<T: Ord> Ord for DedupClause<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub(crate) fn translate_clauses<T>(
    clauses: &mut [DedupClause<T>],
    trans_table: &HashMap<T, T>,
    dedup: bool,
) where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash,
{
    // translate literals and sort and deduplicate literals
    for DedupClause { clause, .. } in clauses.iter_mut() {
        for (l, _) in &mut clause.literals {
            while let Some(trans_l) = trans_table.get(l) {
                *l = *trans_l;
            }
        }
        clause.literals.sort();
        if dedup && clause.kind == ClauseKind::And {
            clause.literals.dedup();
        }
    }
}

// duplicates will be replaced by single-literal clauses with literal to first occurrences
pub(crate) fn deduplicate_clauses<T>(clauses: &mut Vec<DedupClause<T>>) -> HashMap<T, T>
where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    clauses.sort_by_key(
        |DedupClause {
             orig_index: i,
             clause: c,
             ..
         }| (c.kind, c.literals.clone(), *i),
    );

    let mut trans_table = HashMap::<T, T>::new();
    {
        let mut prev: Option<(T, &mut Clause<T>)> = None;
        for DedupClause {
            orig_index: orig_i,
            clause,
            ..
        } in clauses.iter_mut()
        {
            if let Some((prev_orig_i, ref prev_clause)) = prev {
                if **prev_clause == *clause {
                    trans_table.insert(*orig_i, prev_orig_i);
                    continue;
                }
            }
            prev = Some((*orig_i, clause));
        }
    }
    clauses.dedup_by_key(|DedupClause { clause: c, .. }| (c.kind, c.literals.clone()));
    translate_clauses(clauses, &mut trans_table, true);
    clauses.sort();
    trans_table
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

// ALGORITHM to deduplicate literals
// IDEA: deduplicate first 2-literal clauses, next 2-literal clauses and 2-literal-clauses
// from 2-literals with greatest number of occurrences.
// after optimize_clause_circuit is needed!
// algorithm similar to deduplicate_literal_clauses_0 - deduplicate once multiple 2-literals.

// return extra clauses with range of placement.
// argument is clause slice: element: (clause_index, Option<extra_clause_index>, clause)
// extra_clause_index >= input_len + total_clause_num
// if extra_clause_index is not None - new index of new extra clause
// if extra_clause_index is None - old clause
// if clause empty and extra_clause_index is not None -
//    clause_index - original index of removed clause
//    extra_clause_index - index of clause that replace removed clause.
// extra_clause_start - start index for new extra clauses

pub(crate) fn deduplicate_literal_clauses_0<T>(
    extra_clause_start: &mut usize,
    clauses: &mut Vec<DedupClause<T>>,
    trans_table: &mut HashMap<T, T>,
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

    let kind = clauses.first().unwrap().clause.kind;

    let same_occur_lits = {
        let mut lit_clause_tbl = vec![(0, vec![]); *extra_clause_start << 1];
        for (i, (l, _)) in lit_clause_tbl.iter_mut().enumerate() {
            *l = i;
        }
        for (i, DedupClause { clause, .. }) in clauses.iter().enumerate() {
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

    // apply same occurrence literals list (clauses) into clauses
    for (same_lits, occurs) in same_occur_lits.into_iter() {
        if same_lits.len() > 1 {
            let extra_lit = T::try_from(*extra_clause_start).unwrap();
            for occur in &occurs {
                let DedupClause {
                    orig_index, clause, ..
                } = &mut clauses[*occur];
                remove_sorted_ref(&mut clause.literals, &same_lits);
                clause.literals.push((extra_lit, false));
                if clause.literals.len() == 1 {
                    trans_table.insert(*orig_index, clause.literals.first().unwrap().0);
                }
            }
            let dedup_clause = &clauses[*occurs.first().unwrap()];
            let new_orig_index =
                T::try_from(usize::try_from(dedup_clause.orig_index).unwrap() - 1).unwrap();
            clauses.push(DedupClause {
                orig_index: new_orig_index,
                extra_index: Some(extra_lit),
                clause: Clause {
                    kind,
                    literals: same_lits.clone(),
                },
            });
            *extra_clause_start += 1;
        }
    }
    clauses.retain(|x| x.clause.literals.len() != 1);

    // translate literals and sort and deduplicate literals
    translate_clauses(clauses, &trans_table, true);
    clauses.sort()
}

pub(crate) fn deduplicate_literal_clauses<T>(
    extra_clause_start: &mut usize,
    clauses: &mut Vec<DedupClause<T>>,
    trans_table: &mut HashMap<T, T>,
) where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    if clauses.is_empty() {
        return;
    }
    let kind = clauses.first().unwrap().clause.kind;

    //println!("StartDedupLitClauses");
    let total_lit_count = clauses
        .iter()
        .map(|dc| dc.clause.literals.len())
        .sum::<usize>();
    for _ in 0..std::cmp::max(total_lit_count / 20, 100) {
        // get pair_count_map sorted by count descending
        let pairlit_clause_map = {
            let mut pairlit_clause_map = HashMap::<((T, bool), (T, bool)), Vec<usize>>::new();
            for (ci, DedupClause { clause, .. }) in clauses.iter().enumerate() {
                for (i, ls1) in clause.literals.iter().enumerate() {
                    for ls2 in clause.literals[i + 1..].iter().filter(|x| *x != ls1) {
                        let (ls1, ls2) = if ls1 < ls2 { (ls1, ls2) } else { (ls2, ls1) };
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

        //println!("pairlitmap: {:?}", pairlit_clause_map);
        const ITEM_NUM_TO_CHOICE: usize = 10;
        let mut used_clauses = HashSet::<usize>::new();
        let pairlit_clause_map_len = pairlit_clause_map.len();
        let mut pi = 0;
        let mut have_changes = false;
        while pi < pairlit_clause_map_len {
            if pairlit_clause_map[pi].1.len() < 2 {
                pi += 1;
                continue;
            }
            if used_clauses.len() == clauses.len() {
                break;
            }
            // NOTODO: fix between ordering between extra clauses from
            // deduplicate_literal_clauses_0 and this deduplicate_literal_clauses.
            // ^^ unnecessary: because literals in extra clauses from dlc_0 occurred only once!
            // additional (occurs.count, ri) - ri to force choice of first pair in order
            let best_pi = pairlit_clause_map
                [pi..std::cmp::min(pairlit_clause_map_len, pi + ITEM_NUM_TO_CHOICE)]
                .iter()
                .enumerate()
                .map(|(i, (_, occurs))| {
                    (
                        i,
                        (
                            occurs.iter().filter(|x| !used_clauses.contains(x)).count(),
                            // reverse i - to choose first item with max occurrences
                            std::cmp::Reverse(i),
                        ),
                    )
                })
                .max_by_key(|(_, (occur_count, ri))| (*occur_count, *ri))
                .map(|(i, (_, _))| pi + i)
                .unwrap();
            // choose best_pi if occurrence count is greater than 1
            let ((ls1, ls2), occurs) = &pairlit_clause_map[best_pi];
            let real_occurs = occurs
                .into_iter()
                .filter(|x| !used_clauses.contains(x))
                .copied()
                .collect::<Vec<_>>();

            // println!(
            //     "pairlit: {} {} {:?} {:?}",
            //     pi,
            //     best_pi,
            //     (ls1, ls2),
            //     real_occurs
            // );
            if real_occurs.len() >= 2 {
                // process occurrences
                let mut lit1_extra_clause_index = None;
                for occur in &real_occurs {
                    let DedupClause {
                        clause,
                        extra_index,
                        ..
                    } = &clauses[*occur];
                    if clause.literals.len() == 2 {
                        //println!("  old_extra_clauses found: {:?} {:?}", extra_index, clause);
                        lit1_extra_clause_index = *extra_index;
                    }
                }

                let extra_lit = T::try_from(*extra_clause_start).unwrap();
                let same_lits = [*ls1, *ls2];
                // important: find firsrt in original occurrences first orig_index
                //            not in real_occurs!! it must be really first occurrence
                //            with smallest orig_index and smallest clauses index.
                let dedup_clause = &clauses[*occurs.first().unwrap()];
                let new_orig_index = if dedup_clause.extra_index.is_some() {
                    dedup_clause.orig_index
                } else {
                    T::try_from(usize::try_from(dedup_clause.orig_index).unwrap() - 1).unwrap()
                };
                // is_old_extra - if same 2-literal clause as pair literal is exist
                // then do not remove and replace 2-literal clause and just
                // replace original index if needed.
                let is_old_extra = lit1_extra_clause_index.is_some();
                let extra_lit = lit1_extra_clause_index.unwrap_or(extra_lit);

                for occur in &real_occurs {
                    let DedupClause {
                        orig_index, clause, ..
                    } = &mut clauses[*occur];
                    //println!("  clause: {:?}", clause);
                    if is_old_extra && clause.literals.len() == 2 {
                        //*orig_index = std::cmp::min(*orig_index, new_orig_index);
                        assert!(*orig_index <= new_orig_index);
                        continue;
                    }
                    remove_sorted_ref(&mut clause.literals, &same_lits);
                    clause.literals.push((extra_lit, false));
                    // only one 2-literal clause with ls1,ls2)
                    if clause.literals.len() == 1 {
                        trans_table.insert(*orig_index, clause.literals.first().unwrap().0);
                    }
                }

                if !is_old_extra {
                    clauses.push(DedupClause {
                        orig_index: new_orig_index,
                        extra_index: Some(extra_lit),
                        clause: Clause {
                            kind,
                            literals: same_lits.to_vec(),
                        },
                    });
                    *extra_clause_start += 1;
                }
                // add real occurs to used_clauses
                used_clauses.extend(real_occurs);
                have_changes = true;
            } else {
                pi += ITEM_NUM_TO_CHOICE;
                continue;
            }
            pi += 1;
        }

        clauses.retain(|x| x.clause.literals.len() != 1);
        // translate literals and sort and deduplicate literals
        translate_clauses(clauses, &trans_table, true);
        clauses.sort();
        //println!("Clauses: After: {:?}", clauses);

        if !have_changes {
            break;
        }
    }
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

pub(crate) fn join_deduplicates_to_clause_circuit<T>(
    input_len: usize,
    extra_clause_index: usize,
    and_clauses: Vec<DedupClause<T>>,
    and_trans_map: HashMap<T, T>,
    xor_clauses: Vec<DedupClause<T>>,
    xor_trans_map: HashMap<T, T>,
    outputs: &[(T, bool)],
) -> ClauseCircuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut out_clauses = merge_sorted_by_key(and_clauses, xor_clauses, |x| DedupClause {
        orig_index: x.orig_index,
        extra_index: x.extra_index,
        clause: Clause::new_and([]),
    });
    let mut trans_table = vec![T::default(); extra_clause_index];
    for (
        i,
        DedupClause {
            orig_index: j,
            extra_index: extra_j,
            ..
        },
    ) in out_clauses.iter().enumerate()
    {
        let final_lit = T::try_from(i + input_len).unwrap();
        if let Some(ej) = extra_j {
            trans_table[usize::try_from(*ej).unwrap()] = final_lit;
        } else {
            trans_table[usize::try_from(*j).unwrap()] = final_lit;
        }
    }
    for DedupClause { clause, .. } in &mut out_clauses {
        for (l, _) in &mut clause.literals {
            let l_u = usize::try_from(*l).unwrap();
            if l_u >= input_len {
                *l = trans_table[l_u];
            }
        }
    }
    ClauseCircuit::new(
        T::try_from(input_len).unwrap(),
        out_clauses
            .into_iter()
            .map(|DedupClause { clause, .. }| clause)
            .filter(|c| c.len() != 0),
        outputs.iter().map(|(l, n)| {
            let mut out_l = *l;
            while let Some(trans_l) = and_trans_map.get(&out_l) {
                out_l = *trans_l;
            }
            if out_l == *l {
                while let Some(trans_l) = xor_trans_map.get(&out_l) {
                    out_l = *trans_l;
                }
            }
            let l_u = usize::try_from(out_l).unwrap();
            if l_u >= input_len {
                (trans_table[l_u], *n)
            } else {
                (*l, *n)
            }
        }),
    )
    .unwrap()
}

pub(crate) fn check_if_clauses_need_optimization_and_fix<T>(clauses: &mut [DedupClause<T>]) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    for DedupClause { clause, .. } in clauses {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dedup_clause<T>(orig_index: T, extra_index: Option<T>, clause: Clause<T>) -> DedupClause<T> {
        DedupClause {
            orig_index,
            extra_index,
            clause,
        }
    }

    #[test]
    fn test_translate_clauses() {
        let mut clauses = vec![
            dedup_clause(
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false), (9, true)]),
            ),
            dedup_clause(
                9,
                None,
                Clause::new_and([(0, false), (0, false), (7, true), (8, false)]),
            ),
        ];
        translate_clauses(
            &mut clauses,
            &HashMap::from_iter([(7, 30), (8, 31), (9, 32), (31, 33)]),
            true,
        );
        assert_eq!(
            vec![
                dedup_clause(
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false), (32, true)]),
                ),
                dedup_clause(
                    9,
                    None,
                    Clause::new_and([(0, false), (30, true), (33, false)])
                ),
            ],
            clauses
        );

        let mut clauses = vec![
            dedup_clause(
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false), (9, true)]),
            ),
            dedup_clause(
                9,
                None,
                Clause::new_and([(0, false), (0, false), (7, true), (8, false)]),
            ),
        ];
        translate_clauses(
            &mut clauses,
            &HashMap::from_iter([(7, 30), (8, 31), (9, 32), (31, 33)]),
            false,
        );
        assert_eq!(
            vec![
                dedup_clause(
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false), (32, true)]),
                ),
                dedup_clause(
                    9,
                    None,
                    Clause::new_and([(0, false), (0, false), (30, true), (33, false)])
                ),
            ],
            clauses
        );
    }

    #[test]
    fn test_deduplicate_clauses() {
        let mut clauses = vec![
            dedup_clause(
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false)]),
            ),
            dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
            dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
            dedup_clause(
                8,
                None,
                Clause::new_and([(3, true), (4, false), (6, false)]),
            ),
            dedup_clause(6, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert_eq!(
            HashMap::from_iter([(6, 5)]),
            deduplicate_clauses(&mut clauses)
        );
        assert_eq!(
            vec![
                dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
                dedup_clause(
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false)])
                ),
                dedup_clause(
                    8,
                    None,
                    Clause::new_and([(3, true), (4, false), (5, false)])
                ),
            ],
            clauses
        );

        let mut clauses = vec![
            dedup_clause(
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false)]),
            ),
            dedup_clause(5, None, Clause::new_and([(0, false), (1, true)])),
            dedup_clause(4, None, Clause::new_and([(0, false), (2, true)])),
            dedup_clause(
                8,
                None,
                Clause::new_and([(3, true), (5, false), (6, false)]),
            ),
            dedup_clause(6, None, Clause::new_and([(0, false), (2, true)])),
            dedup_clause(9, None, Clause::new_and([(0, false), (2, true)])),
            dedup_clause(
                10,
                None,
                Clause::new_and([(1, true), (2, false), (9, false)]),
            ),
        ];
        assert_eq!(
            HashMap::from_iter([(6, 4), (9, 4)]),
            deduplicate_clauses(&mut clauses)
        );
        assert_eq!(
            vec![
                dedup_clause(4, None, Clause::new_and([(0, false), (2, true)])),
                dedup_clause(5, None, Clause::new_and([(0, false), (1, true)])),
                dedup_clause(
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false)])
                ),
                dedup_clause(
                    8,
                    None,
                    Clause::new_and([(3, true), (4, false), (5, false)])
                ),
                dedup_clause(
                    10,
                    None,
                    Clause::new_and([(1, true), (2, false), (4, false)]),
                ),
            ],
            clauses
        );

        let mut clauses = vec![
            dedup_clause(
                7,
                None,
                Clause::new_and([(1, true), (3, false), (5, false)]),
            ),
            dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
            dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
            dedup_clause(6, None, Clause::new_xor([(0, false), (2, true)])),
            dedup_clause(
                8,
                None,
                Clause::new_and([(3, true), (4, false), (6, false)]),
            ),
        ];
        assert!(deduplicate_clauses(&mut clauses).is_empty());
        assert_eq!(
            vec![
                dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
                dedup_clause(6, None, Clause::new_xor([(0, false), (2, true)])),
                dedup_clause(
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (5, false)])
                ),
                dedup_clause(
                    8,
                    None,
                    Clause::new_and([(3, true), (4, false), (6, false)])
                )
            ],
            clauses
        );

        // link two duplicates to some clause. and remove one.
        let mut clauses = vec![
            dedup_clause(
                7,
                None,
                Clause::new_and([(1, true), (3, false), (4, false)]),
            ),
            dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
            dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
            dedup_clause(
                8,
                None,
                Clause::new_and([(3, true), (5, false), (6, false)]),
            ),
            dedup_clause(6, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert_eq!(
            HashMap::from_iter([(6, 5)]),
            deduplicate_clauses(&mut clauses)
        );
        assert_eq!(
            vec![
                dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
                dedup_clause(
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (4, false)])
                ),
                dedup_clause(8, None, Clause::new_and([(3, true), (5, false)]))
            ],
            clauses
        );

        // link two duplicates to some clause. and do not remove one because is xor.
        let mut clauses = vec![
            dedup_clause(
                7,
                None,
                Clause::new_xor([(1, true), (3, false), (4, false)]),
            ),
            dedup_clause(4, None, Clause::new_xor([(0, false), (1, true)])),
            dedup_clause(5, None, Clause::new_xor([(0, false), (2, true)])),
            dedup_clause(
                8,
                None,
                Clause::new_xor([(3, true), (5, false), (6, false)]),
            ),
            dedup_clause(6, None, Clause::new_xor([(0, false), (2, true)])),
        ];
        assert_eq!(
            HashMap::from_iter([(6, 5)]),
            deduplicate_clauses(&mut clauses)
        );
        assert_eq!(
            vec![
                dedup_clause(4, None, Clause::new_xor([(0, false), (1, true)])),
                dedup_clause(5, None, Clause::new_xor([(0, false), (2, true)])),
                dedup_clause(
                    7,
                    None,
                    Clause::new_xor([(1, true), (3, false), (4, false)])
                ),
                dedup_clause(
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
            dedup_clause(
                7,
                None,
                Clause::new_and([(1, true), (3, false), (4, false)]),
            ),
            dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
            dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
            dedup_clause(8, None, Clause::new_and([(3, true), (5, false), (6, true)])),
            dedup_clause(6, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert_eq!(
            HashMap::from_iter([(6, 5)]),
            deduplicate_clauses(&mut clauses)
        );
        assert_eq!(
            vec![
                dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
                dedup_clause(
                    7,
                    None,
                    Clause::new_and([(1, true), (3, false), (4, false)])
                ),
                dedup_clause(8, None, Clause::new_and([(3, true), (5, false), (5, true)]))
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
                10,
                vec![
                    dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                    dedup_clause(4, Some(8), Clause::new_and([(0, false), (3, true)])),
                    dedup_clause(
                        5,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (8, false)])
                    ),
                ],
                HashMap::new(),
                vec![
                    dedup_clause(6, None, Clause::new_xor([(0, false), (3, true)])),
                    dedup_clause(6, Some(9), Clause::new_xor([(0, false), (2, true)])),
                    dedup_clause(
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (6, false), (9, false)])
                    ),
                ],
                HashMap::new(),
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
                10,
                vec![
                    dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                    dedup_clause(4, Some(8), Clause::new_and([(0, false), (3, true)])),
                    dedup_clause(
                        6,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (8, false)])
                    ),
                ],
                HashMap::new(),
                vec![
                    dedup_clause(5, None, Clause::new_xor([(0, false), (3, true)])),
                    dedup_clause(5, Some(9), Clause::new_xor([(0, false), (2, true)])),
                    dedup_clause(
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (5, false), (9, false)])
                    ),
                ],
                HashMap::new(),
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
                10,
                vec![
                    dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                    dedup_clause(
                        4,
                        Some(8),
                        Clause::new_and([(0, false), (3, true), (4, false)])
                    ),
                    dedup_clause(
                        6,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (8, false)])
                    ),
                ],
                HashMap::new(),
                vec![
                    dedup_clause(5, None, Clause::new_xor([(0, false), (3, true)])),
                    dedup_clause(
                        5,
                        Some(9),
                        Clause::new_xor([(0, false), (2, true), (5, true)])
                    ),
                    dedup_clause(
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (5, false), (9, false)])
                    ),
                ],
                HashMap::new(),
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
                12,
                vec![
                    dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                    dedup_clause(
                        4,
                        Some(8),
                        Clause::new_and([(0, false), (3, true), (4, false)])
                    ),
                    dedup_clause(4, Some(10), Clause::new_and([(0, false), (8, false)])),
                    dedup_clause(
                        6,
                        None,
                        Clause::new_and([(1, true), (2, true), (4, false), (10, false)])
                    ),
                ],
                HashMap::new(),
                vec![
                    dedup_clause(5, None, Clause::new_xor([(0, false), (3, true)])),
                    dedup_clause(
                        5,
                        Some(9),
                        Clause::new_xor([(0, false), (2, true), (5, true)])
                    ),
                    dedup_clause(5, Some(11), Clause::new_xor([(2, true), (9, true)])),
                    dedup_clause(
                        7,
                        None,
                        Clause::new_xor([(1, true), (3, true), (5, false), (11, false)])
                    ),
                ],
                HashMap::new(),
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
                12,
                vec![
                    dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                    dedup_clause(6, None, Clause::new_and([(0, false), (2, true)])),
                    dedup_clause(
                        6,
                        Some(10),
                        Clause::new_and([(0, false), (3, true), (4, false)])
                    ),
                    dedup_clause(
                        8,
                        None,
                        Clause::new_and([(1, true), (2, true), (6, false), (10, false)])
                    ),
                ],
                HashMap::new(),
                vec![
                    dedup_clause(5, None, Clause::new_xor([(0, false), (3, true)])),
                    dedup_clause(7, None, Clause::new_xor([(1, false), (3, true)])),
                    dedup_clause(
                        7,
                        Some(11),
                        Clause::new_xor([(0, false), (2, true), (5, true)])
                    ),
                    dedup_clause(
                        9,
                        None,
                        Clause::new_xor([(1, true), (3, true), (7, false), (11, false)])
                    ),
                ],
                HashMap::new(),
                &[(8, false), (9, false)]
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
                [(10, false), (11, false), (6, false), (5, false)]
            )
            .unwrap(),
            join_deduplicates_to_clause_circuit(
                4,
                12,
                vec![
                    dedup_clause(4, None, Clause::new_and([(0, false), (1, true)])),
                    dedup_clause(6, None, Clause::new_and([(0, false), (2, true)])),
                    dedup_clause(
                        6,
                        Some(10),
                        Clause::new_and([(0, false), (3, true), (4, false)])
                    ),
                    dedup_clause(
                        8,
                        None,
                        Clause::new_and([(1, true), (2, true), (6, false), (10, false)])
                    ),
                ],
                HashMap::from_iter([(12, 6)]),
                vec![
                    dedup_clause(5, None, Clause::new_xor([(0, false), (3, true)])),
                    dedup_clause(7, None, Clause::new_xor([(1, false), (3, true)])),
                    dedup_clause(
                        7,
                        Some(11),
                        Clause::new_xor([(0, false), (2, true), (5, true)])
                    ),
                    dedup_clause(
                        9,
                        None,
                        Clause::new_xor([(1, true), (3, true), (7, false), (11, false)])
                    ),
                ],
                HashMap::from_iter([(13, 5)]),
                &[(8, false), (9, false), (12, false), (13, false)]
            )
        );
    }

    #[test]
    fn test_check_if_clauses_need_optimization_and_fix() {
        let mut clauses = vec![
            dedup_clause(
                4,
                None,
                Clause::new_and([(0, false), (1, true), (2, false)]),
            ),
            dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(!check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                dedup_clause(
                    4,
                    None,
                    Clause::new_and([(0, false), (1, true), (2, false)])
                ),
                dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            dedup_clause(
                4,
                None,
                Clause::new_and([(0, false), (1, true), (1, false)]),
            ),
            dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                dedup_clause(
                    4,
                    None,
                    Clause::new_and([(0, false), (1, true), (1, false)])
                ),
                dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            dedup_clause(4, None, Clause::new_xor([(0, false), (1, true), (1, true)])),
            dedup_clause(5, None, Clause::new_xor([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                dedup_clause(4, None, Clause::new_xor([(0, false), (1, true), (1, true)])),
                dedup_clause(5, None, Clause::new_xor([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            dedup_clause(4, None, Clause::new_and([(0, true)])),
            dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                dedup_clause(4, None, Clause::new_and([(0, true), (0, true)])),
                dedup_clause(5, None, Clause::new_and([(0, false), (2, true)])),
            ]
        );

        let mut clauses = vec![
            dedup_clause(4, None, Clause::new_xor([(0, true)])),
            dedup_clause(5, None, Clause::new_xor([(0, false), (2, true)])),
        ];
        assert!(check_if_clauses_need_optimization_and_fix(&mut clauses));
        assert_eq!(
            clauses,
            vec![
                dedup_clause(4, None, Clause::new_and([(0, true), (0, true)])),
                dedup_clause(5, None, Clause::new_xor([(0, false), (2, true)])),
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
    fn test_deduplicate_literal_clauses_0() {
        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_and([
                    (3, false), // 2 (c1, c2, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (5, false), // 1 (c0, c3, c4)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses_0(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(14, 30), (12, 31)]), trans_map);
        assert_eq!(extra_clause_index, 33);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (5, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (6, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (4, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false), (31, false), (32, false)]
                    }
                }
            ],
            clauses,
        );

        // different orig_index
        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                17,
                None,
                Clause::new_and([
                    (3, false), // 2 (c1, c2, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                22,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (5, false), // 1 (c0, c3, c4)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                25,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses_0(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(25, 30), (17, 31)]), trans_map);
        assert_eq!(33, extra_clause_index);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (5, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (6, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (4, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false), (31, false), (32, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (1, true),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (1, true),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_and([
                    (3, false), // 2 (c1, c2, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (1, false), // 1 (c0, c3, c4)
                    (1, true),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (5, false), // 1 (c0, c3, c4)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses_0(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(14, 30), (12, 31)]), trans_map);
        assert_eq!(33, extra_clause_index);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (5, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, true), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (6, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (4, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, true), (31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, true), (30, false), (31, false), (32, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_and([
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (1, false), // 1 (c0, c3, c4)
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (5, false), // 1 (c0, c3, c4)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses_0(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::new(), trans_map);
        assert_eq!(33, extra_clause_index);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (5, false)],
                    },
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (6, false), (7, false)],
                    },
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (4, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(30, false), (31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4) // block deduplication!
                    (2, false),
                    (3, false), // 2 (c1, c2, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (1, false), // 1 (c0, c3, c4)
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c1, c3)
                    (5, false), // 1 (c0, c3, c4)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses_0(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::new(), trans_map);
        assert_eq!(32, extra_clause_index);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false), (5, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (6, false), (7, false)],
                    },
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (4, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (5, false), (30, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false), (5, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (0, false), // 3 (c0, c1, c3)
                    (2, false), // 3 (c0, c1, c3)
                    (4, false), // 3 (c0, c1, c3)
                ]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([
                    (0, false), // 3 (c0, c1, c3)
                    (2, false), // 3 (c0, c1, c3)
                    (3, false), // 2 (c1, c2, c3)
                    (4, false), // 3 (c0, c1, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_and([
                    (3, false), // 2 (c1, c2, c3)
                    (6, false), // 2 (c1, c2, c3)
                    (7, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses_0(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(12, 31), (10, 30)]), trans_map);
        assert_eq!(32, extra_clause_index);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (2, false), (4, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (6, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(30, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (5, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (2, false),
                    (3, false),  // 2 (c1, c2, c3)
                    (4, false),  // 3 (c1, c3)
                    (6, false),  // 2 (c1, c2, c3)
                    (10, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_and([
                    (3, false),  // 2 (c1, c2, c3)
                    (6, false),  // 2 (c1, c2, c3)
                    (10, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_and([
                    (0, false), // 3 (c1, c3)
                    (1, false), // 1 (c0, c3, c4)
                    (2, false),
                    (3, false),  // 2 (c1, c2, c3)
                    (4, false),  // 3 (c1, c3)
                    (5, false),  // 1 (c0, c3, c4)
                    (6, false),  // 2 (c1, c2, c3)
                    (10, false), // 2 (c1, c2, c3)
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([
                    (1, false), // 1 (c0, c3, c4)
                    (5, false), // 1 (c0, c3, c4)
                ]),
            ),
            dedup_clause(
                15,
                None,
                Clause::new_and([
                    (7, false),
                    (8, false),  // 4 (c15, c16)
                    (11, false), // 4 (c15, c16)
                    (12, false), // 4 (c15, c16)
                ]),
            ),
            dedup_clause(
                16,
                None,
                Clause::new_and([
                    (3, false),  // 2 (c1, c2, c3)
                    (6, false),  // 2 (c1, c2, c3)
                    (8, false),  // 4 (c15, c16)
                    (10, false), // 2 (c1, c2, c3)
                    (11, false), // 4 (c15, c16)
                    (12, false), // 4 (c15, c16)
                ]),
            ),
            dedup_clause(
                17,
                None,
                Clause::new_and([(7, false), (9, false), (16, false)]),
            ),
            dedup_clause(18, None, Clause::new_and([(9, false), (16, false)])),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses_0(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(
            HashMap::from_iter([(14, 30), (12, 31), (18, 34)]),
            trans_map
        );
        assert_eq!(35, extra_clause_index);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (5, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (6, false), (10, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (4, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (30, false), (31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: Some(33),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (11, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 15,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (33, false)]
                    }
                },
                DedupClause {
                    orig_index: 16,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(31, false), (33, false)]
                    }
                },
                DedupClause {
                    orig_index: 16,
                    extra_index: Some(34),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(9, false), (16, false)]
                    }
                },
                DedupClause {
                    orig_index: 17,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (34, false)]
                    }
                }
            ],
            clauses,
        );
    }

    #[test]
    fn test_dedup_clause_ordering() {
        assert!(
            dedup_clause(4, None, Clause::new_and([])) < dedup_clause(5, None, Clause::new_and([]))
        );
        assert!(
            dedup_clause(4, None, Clause::new_and([]))
                < dedup_clause(4, Some(10), Clause::new_and([]))
        );
        assert!(
            dedup_clause(4, Some(10), Clause::new_and([]))
                < dedup_clause(4, Some(11), Clause::new_and([]))
        );
    }

    #[test]
    fn test_deduplicate_literal_clauses() {
        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (3, false),
                    (5, false),
                    (7, false),
                    (8, false),
                ]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([(1, false), (3, false), (5, false), (7, false), (8, false)]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_and([(1, false), (3, false), (7, false), (8, false)]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_and([(1, false), (3, false), (7, false)]),
            ),
            dedup_clause(14, None, Clause::new_and([(1, false), (7, false)])),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(
            HashMap::from_iter([(14, 30), (12, 32), (11, 33), (13, 31)]),
            trans_map
        );
        assert_eq!(extra_clause_index, 34);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(33),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (33, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(10, None, Clause::new_and([(1, false), (2, false)])),
            dedup_clause(11, None, Clause::new_and([(2, false), (3, false)])),
            dedup_clause(
                12,
                None,
                Clause::new_and([(1, false), (2, false), (4, false)]),
            ),
            dedup_clause(13, None, Clause::new_and([(2, false), (5, false)])),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(10, 30)]), trans_map);
        assert_eq!(extra_clause_index, 31);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (3, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (5, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(10, None, Clause::new_and([(1, false), (2, false)])),
            dedup_clause(11, None, Clause::new_and([(2, false), (2, true)])),
            dedup_clause(
                12,
                None,
                Clause::new_and([(1, false), (2, false), (4, false)]),
            ),
            dedup_clause(13, None, Clause::new_and([(2, false), (5, true)])),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(10, 30)]), trans_map);
        assert_eq!(extra_clause_index, 31);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (2, true)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (5, true)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(10, None, Clause::new_and([(1, false), (2, false)])),
            dedup_clause(11, None, Clause::new_and([(2, false), (3, false)])),
            dedup_clause(
                12,
                None,
                Clause::new_and([(1, false), (2, false), (4, false)]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_and([(2, false), (3, false), (5, false), (6, false)]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([(2, false), (4, false), (6, false)]),
            ),
            dedup_clause(
                15,
                None,
                Clause::new_and([(5, false), (6, false), (7, false)]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(11, 31), (10, 30)]), trans_map);
        assert_eq!(extra_clause_index, 33);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (3, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (6, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(31, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (4, false), (6, false)]
                    }
                },
                DedupClause {
                    orig_index: 15,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (32, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([(1, false), (2, false), (3, false), (5, false)]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([(1, false), (3, false), (5, false)]),
            ),
            dedup_clause(12, None, Clause::new_and([(3, false), (5, false)])),
            dedup_clause(
                13,
                None,
                Clause::new_and([(6, false), (7, false), (8, false), (9, false)]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([(6, false), (7, false), (8, false)]),
            ),
            dedup_clause(15, None, Clause::new_and([(6, false), (7, false)])),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(
            HashMap::from_iter([(14, 33), (12, 30), (15, 31), (11, 32)]),
            trans_map
        );
        assert_eq!(extra_clause_index, 34);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (5, false)]
                    }
                },
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(6, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: Some(33),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(9, false), (33, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_and([(1, false), (3, false), (7, false), (8, false)]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_and([(1, false), (3, false), (5, false), (7, false), (8, false)]),
            ),
            dedup_clause(12, None, Clause::new_and([(1, false), (7, false)])),
            dedup_clause(
                13,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (3, false),
                    (5, false),
                    (7, false),
                    (8, false),
                ]),
            ),
            dedup_clause(
                14,
                None,
                Clause::new_and([(1, false), (3, false), (7, false)]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(
            HashMap::from_iter([(12, 30), (14, 31), (10, 32), (11, 33)]),
            trans_map
        );
        assert_eq!(extra_clause_index, 34);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(33),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (33, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(10, None, Clause::new_and([(1, false), (7, false)])),
            dedup_clause(
                13,
                None,
                Clause::new_and([(1, false), (3, false), (7, false)]),
            ),
            dedup_clause(
                15,
                None,
                Clause::new_and([(1, false), (3, false), (7, false), (8, false)]),
            ),
            dedup_clause(
                16,
                None,
                Clause::new_and([(1, false), (3, false), (5, false), (7, false), (8, false)]),
            ),
            dedup_clause(
                19,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (3, false),
                    (5, false),
                    (7, false),
                    (8, false),
                ]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(
            HashMap::from_iter([(13, 31), (16, 33), (15, 32), (10, 30)]),
            trans_map
        );
        assert_eq!(extra_clause_index, 34);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (7, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (30, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (31, false)]
                    }
                },
                DedupClause {
                    orig_index: 15,
                    extra_index: Some(33),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (32, false)]
                    }
                },
                DedupClause {
                    orig_index: 19,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (33, false)]
                    }
                }
            ],
            clauses,
        );

        // with older extra clauses
        let mut clauses = vec![
            dedup_clause(
                19,
                Some(38),
                Clause::new_and([(0, false), (3, false), (12, false)]),
            ),
            dedup_clause(20, None, Clause::new_and([(1, false), (38, false)])),
            dedup_clause(
                22,
                Some(37),
                Clause::new_and([(4, false), (5, false), (11, false)]),
            ),
            dedup_clause(
                23,
                None,
                Clause::new_and([(1, false), (37, false), (38, false)]),
            ),
            dedup_clause(
                25,
                None,
                Clause::new_and([(1, false), (8, false), (37, false), (38, false)]),
            ),
            dedup_clause(
                25,
                Some(35),
                Clause::new_and([(6, false), (7, false), (9, false)]),
            ),
            dedup_clause(
                26,
                None,
                Clause::new_and([
                    (1, false),
                    (8, false),
                    (35, false),
                    (37, false),
                    (38, false),
                ]),
            ),
            dedup_clause(
                29,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (8, false),
                    (35, false),
                    (37, false),
                    (38, false),
                ]),
            ),
        ];
        let mut extra_clause_index = 40;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(
            HashMap::from_iter([(23, 41), (26, 43), (25, 42), (20, 40)]),
            trans_map
        );
        assert_eq!(extra_clause_index, 44);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 19,
                    extra_index: Some(38),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (3, false), (12, false)]
                    }
                },
                DedupClause {
                    orig_index: 19,
                    extra_index: Some(40),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (38, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: Some(37),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (5, false), (11, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: Some(41),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(37, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 24,
                    extra_index: Some(42),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (41, false)]
                    }
                },
                DedupClause {
                    orig_index: 25,
                    extra_index: Some(35),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(6, false), (7, false), (9, false)]
                    }
                },
                DedupClause {
                    orig_index: 25,
                    extra_index: Some(43),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(35, false), (42, false)]
                    }
                },
                DedupClause {
                    orig_index: 29,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (43, false)]
                    }
                }
            ],
            clauses,
        );

        // important testcase: old extra clause reordering avoiding.
        let mut clauses = vec![
            dedup_clause(20, None, Clause::new_and([(5, false), (6, false)])),
            dedup_clause(
                21,
                None,
                Clause::new_and([(5, false), (6, false), (8, false), (11, false)]),
            ),
            dedup_clause(
                22,
                None,
                Clause::new_and([(1, false), (2, false), (3, false)]),
            ),
            dedup_clause(
                23,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (3, false),
                    (4, false),
                    (5, false),
                    (6, false),
                ]),
            ),
            dedup_clause(
                24,
                None,
                Clause::new_and([(1, false), (2, false), (4, false), (5, false), (6, false)]),
            ),
            dedup_clause(
                25,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (4, false),
                    (5, false),
                    (6, false),
                    (7, false),
                ]),
            ),
            dedup_clause(
                26,
                None,
                Clause::new_and([(1, false), (2, false), (8, false)]),
            ),
        ];
        let mut extra_clause_index = 40;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(20, 41), (24, 43)]), trans_map);
        assert_eq!(extra_clause_index, 44);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 19,
                    extra_index: Some(41),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (6, false)]
                    }
                },
                DedupClause {
                    orig_index: 21,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (11, false), (41, false)]
                    }
                },
                DedupClause {
                    orig_index: 21,
                    extra_index: Some(40),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: Some(42),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: Some(43),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(41, false), (42, false)]
                    }
                },
                DedupClause {
                    orig_index: 23,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (43, false)]
                    }
                },
                DedupClause {
                    orig_index: 25,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (43, false)]
                    }
                },
                DedupClause {
                    orig_index: 26,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (40, false)]
                    }
                }
            ],
            clauses,
        );

        // important testcase: old extra clause reordering avoiding.
        let mut clauses = vec![
            dedup_clause(
                20,
                None,
                Clause::new_and([(1, false), (2, false), (3, false)]),
            ),
            dedup_clause(
                21,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (3, false),
                    (4, false),
                    (5, false),
                    (6, false),
                ]),
            ),
            dedup_clause(
                22,
                None,
                Clause::new_and([(1, false), (2, false), (4, false), (5, false), (6, false)]),
            ),
            dedup_clause(
                23,
                None,
                Clause::new_and([
                    (1, false),
                    (2, false),
                    (4, false),
                    (5, false),
                    (6, false),
                    (7, false),
                ]),
            ),
            dedup_clause(24, None, Clause::new_and([(5, false), (6, false)])),
            dedup_clause(
                25,
                None,
                Clause::new_and([(5, false), (6, false), (8, false), (11, false)]),
            ),
            dedup_clause(
                26,
                None,
                Clause::new_and([(1, false), (2, false), (8, false)]),
            ),
        ];
        let mut extra_clause_index = 40;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(24, 41), (22, 43)]), trans_map);
        assert_eq!(extra_clause_index, 44);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 19,
                    extra_index: Some(40),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    }
                },
                DedupClause {
                    orig_index: 20,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 20,
                    extra_index: Some(41),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (6, false)]
                    }
                },
                DedupClause {
                    orig_index: 20,
                    extra_index: Some(42),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 20,
                    extra_index: Some(43),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(41, false), (42, false)]
                    }
                },
                DedupClause {
                    orig_index: 21,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (43, false)]
                    }
                },
                DedupClause {
                    orig_index: 23,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (43, false)]
                    }
                },
                DedupClause {
                    orig_index: 25,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (11, false), (41, false)]
                    }
                },
                DedupClause {
                    orig_index: 26,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (40, false)]
                    }
                }
            ],
            clauses,
        );

        let mut clauses = vec![
            dedup_clause(
                20,
                None,
                Clause::new_and([(1, false), (2, false), (3, false)]),
            ),
            dedup_clause(
                21,
                None,
                Clause::new_and([(2, false), (3, false), (4, false)]),
            ),
            dedup_clause(
                22,
                None,
                Clause::new_and([(2, false), (3, false), (5, false)]),
            ),
            dedup_clause(
                23,
                None,
                Clause::new_and([(2, false), (3, false), (4, false), (6, false)]),
            ),
            dedup_clause(
                24,
                None,
                Clause::new_and([(2, false), (3, false), (7, false)]),
            ),
            dedup_clause(
                25,
                None,
                Clause::new_and([(2, false), (3, false), (8, false)]),
            ),
            dedup_clause(
                26,
                None,
                Clause::new_and([(3, false), (7, false), (8, false)]),
            ),
            dedup_clause(
                27,
                None,
                Clause::new_and([(3, false), (5, false), (9, false)]),
            ),
        ];
        let mut extra_clause_index = 40;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::from_iter([(21, 41)]), trans_map);
        assert_eq!(extra_clause_index, 42);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 19,
                    extra_index: Some(40),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (3, false)]
                    }
                },
                DedupClause {
                    orig_index: 20,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 20,
                    extra_index: Some(41),
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 23,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(6, false), (41, false)]
                    }
                },
                DedupClause {
                    orig_index: 24,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 25,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (40, false)]
                    }
                },
                DedupClause {
                    orig_index: 26,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (7, false), (8, false)]
                    }
                },
                DedupClause {
                    orig_index: 27,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (5, false), (9, false)]
                    }
                }
            ],
            clauses,
        );

        // bintree
        let mut clauses = vec![
            dedup_clause(
                10,
                None,
                Clause::new_xor([(0, false), (1, false), (2, false), (3, false)]),
            ),
            dedup_clause(
                11,
                None,
                Clause::new_xor([(0, true), (1, false), (2, false), (3, false)]),
            ),
            dedup_clause(
                12,
                None,
                Clause::new_xor([(0, false), (1, true), (2, false), (3, false)]),
            ),
            dedup_clause(
                13,
                None,
                Clause::new_xor([(0, true), (1, true), (2, false), (3, false)]),
            ),
            dedup_clause(
                15,
                None,
                Clause::new_xor([(0, false), (1, false), (2, true), (3, false)]),
            ),
            dedup_clause(
                16,
                None,
                Clause::new_xor([(0, true), (1, false), (2, true), (3, false)]),
            ),
            dedup_clause(
                17,
                None,
                Clause::new_xor([(0, false), (1, true), (2, true), (3, false)]),
            ),
            dedup_clause(
                18,
                None,
                Clause::new_xor([(0, true), (1, true), (2, true), (3, false)]),
            ),
            dedup_clause(
                20,
                None,
                Clause::new_xor([(0, false), (1, false), (2, false), (3, true)]),
            ),
            dedup_clause(
                21,
                None,
                Clause::new_xor([(0, true), (1, false), (2, false), (3, true)]),
            ),
            dedup_clause(
                22,
                None,
                Clause::new_xor([(0, false), (1, true), (2, false), (3, true)]),
            ),
            dedup_clause(
                23,
                None,
                Clause::new_xor([(0, true), (1, true), (2, false), (3, true)]),
            ),
            dedup_clause(
                25,
                None,
                Clause::new_xor([(0, false), (1, false), (2, true), (3, true)]),
            ),
            dedup_clause(
                26,
                None,
                Clause::new_xor([(0, true), (1, false), (2, true), (3, true)]),
            ),
            dedup_clause(
                27,
                None,
                Clause::new_xor([(0, false), (1, true), (2, true), (3, true)]),
            ),
            dedup_clause(
                28,
                None,
                Clause::new_xor([(0, true), (1, true), (2, true), (3, true)]),
            ),
        ];
        let mut extra_clause_index = 30;
        let mut trans_map = HashMap::new();
        deduplicate_literal_clauses(&mut extra_clause_index, &mut clauses, &mut trans_map);
        assert_eq!(HashMap::new(), trans_map);
        assert_eq!(extra_clause_index, 38);
        assert_eq!(
            vec![
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(30),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(0, false), (1, false)]
                    }
                },
                DedupClause {
                    orig_index: 9,
                    extra_index: Some(34),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, false), (3, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(30, false), (34, false)]
                    }
                },
                DedupClause {
                    orig_index: 10,
                    extra_index: Some(32),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(0, true), (1, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(32, false), (34, false)]
                    }
                },
                DedupClause {
                    orig_index: 11,
                    extra_index: Some(31),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(0, false), (1, true)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(31, false), (34, false)]
                    }
                },
                DedupClause {
                    orig_index: 12,
                    extra_index: Some(33),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(0, true), (1, true)]
                    }
                },
                DedupClause {
                    orig_index: 13,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(33, false), (34, false)]
                    }
                },
                DedupClause {
                    orig_index: 14,
                    extra_index: Some(36),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, true), (3, false)]
                    }
                },
                DedupClause {
                    orig_index: 15,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(30, false), (36, false)]
                    }
                },
                DedupClause {
                    orig_index: 16,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(32, false), (36, false)]
                    }
                },
                DedupClause {
                    orig_index: 17,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(31, false), (36, false)]
                    }
                },
                DedupClause {
                    orig_index: 18,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(33, false), (36, false)]
                    }
                },
                DedupClause {
                    orig_index: 19,
                    extra_index: Some(35),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, false), (3, true)]
                    }
                },
                DedupClause {
                    orig_index: 20,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(30, false), (35, false)]
                    }
                },
                DedupClause {
                    orig_index: 21,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(32, false), (35, false)]
                    }
                },
                DedupClause {
                    orig_index: 22,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(31, false), (35, false)]
                    }
                },
                DedupClause {
                    orig_index: 23,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(33, false), (35, false)]
                    }
                },
                DedupClause {
                    orig_index: 24,
                    extra_index: Some(37),
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, true), (3, true)]
                    }
                },
                DedupClause {
                    orig_index: 25,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(30, false), (37, false)]
                    }
                },
                DedupClause {
                    orig_index: 26,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(32, false), (37, false)]
                    }
                },
                DedupClause {
                    orig_index: 27,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(31, false), (37, false)]
                    }
                },
                DedupClause {
                    orig_index: 28,
                    extra_index: None,
                    clause: Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(33, false), (37, false)]
                    }
                }
            ],
            clauses,
        );
    }
}
