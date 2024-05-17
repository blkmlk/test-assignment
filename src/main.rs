use std::thread;

const THRESHOLD: usize = 5;

fn main() {}

fn handle_tasks_v1<
    T: Clone + Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> R + Clone + Send + 'static,
>(
    input: Vec<T>,
    f: F,
) -> Vec<R> {
    if input.len() <= THRESHOLD {
        return input.into_iter().map(f).collect();
    }

    let mut result = Vec::with_capacity(input.len());
    let mut joins = vec![];
    for chunk in input.chunks(THRESHOLD) {
        let f = f.clone();
        let chunk: Vec<T> = chunk.to_vec();

        let join = thread::spawn(move || chunk.into_iter().map(|x: T| f(x)).collect::<Vec<R>>());

        joins.push(join);
    }

    for join in joins {
        result.extend(join.join().unwrap());
    }

    result
}

fn handle_tasks_v2<T: Send + 'static, R: Send + 'static, F: Fn(T) -> R + Clone + Send + 'static>(
    input: Vec<T>,
    f: F,
) -> Vec<R> {
    if input.len() <= THRESHOLD {
        return input.into_iter().map(f).collect();
    }

    let (in_tx, in_rx) = crossbeam_channel::bounded(10);
    let (out_tx, out_rx) = crossbeam_channel::bounded(10);

    let mut joins = vec![];
    for _ in 0..=input.len() / THRESHOLD {
        let rx = in_rx.clone();
        let tx = out_tx.clone();
        let f = f.clone();

        let join = thread::spawn(move || {
            for (i, t) in rx {
                tx.send((i, f(t))).unwrap();
            }
        });

        joins.push(join);
    }
    drop(out_tx);

    let mut result = Vec::with_capacity(input.len());
    unsafe {result.set_len(input.len())};

    let tx_join = thread::spawn(move || {
        for (i, t) in input.into_iter().enumerate() {
            in_tx.send((i, t)).unwrap();
        }
        drop(in_tx);
    });

    for (i, r) in out_rx {
        result[i] = r;
    }

    tx_join.join().unwrap();
    for join in joins {
        join.join().unwrap()
    }

    result
}

#[cfg(test)]
mod test {
    use crate::{handle_tasks_v1, handle_tasks_v2};

    #[test]
    fn test_handle_tasks_v1_part_1() {
        let f = |x: i32| -> i32 { x + 1 };

        assert_eq!(handle_tasks_v1(vec![1, 2, 3], f), vec![2, 3, 4]);
        assert_eq!(handle_tasks_v1(vec![], f), vec![]);
        assert_eq!(
            handle_tasks_v1(vec![1, 2, 3, 4, 5, 6, 7], f),
            vec![2, 3, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn test_handle_tasks_v2_part_1() {
        let f = |x: i32| -> i32 { x + 1 };

        assert_eq!(handle_tasks_v2(vec![1, 2, 3], f), vec![2, 3, 4]);
        assert_eq!(handle_tasks_v2(vec![], f), vec![]);
        assert_eq!(
            handle_tasks_v2(vec![1, 2, 3, 4, 5, 6, 7], f),
            vec![2, 3, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn test_handle_tasks_part_2() {
        const K: u64 = 8;
        let f = |x: u64| -> u64 {
            if x == 1 {
                return 0;
            }

            let mut n = x;
            for i in 1..=K {
                if n % 2 == 0 {
                    n /= 2;
                } else {
                    n = n * 3 + 1
                }

                if n == 1 {
                    return i;
                }
            }

            n
        };

        assert_eq!(handle_tasks_v2(vec![], f), vec![]);
        assert_eq!(handle_tasks_v2(vec![0], f), vec![0]);
        assert_eq!(handle_tasks_v2(vec![1, 2, 3, 100], f), vec![0, 1, 7, 88]);

        ()
    }
}
