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

#[cfg(test)]
mod tests {
    use super::*;

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
}
