use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    
    // TODO: implement parallel map!
    let (args_sender, args_receiver) = crossbeam_channel::unbounded();
    let (result_sender, result_receiver) = crossbeam_channel::unbounded();

    let mut threads = Vec::new();
    // spawn worker threads
    for _ in 0..num_threads {
        let args_receiver = args_receiver.clone();
        let result_sender = result_sender.clone();
        threads.push(thread::spawn(move || {
            while let Ok(next_pair) = args_receiver.recv() {
                let (index, value) = next_pair; 
                result_sender.send((index, f(value))).expect("Sending computed result error.");
            }
        }));
    }

    // send args with index
    let input_vec_len = input_vec.len();
    for index in 0..input_vec_len {
        args_sender
            .send((input_vec_len-index-1, input_vec.pop().unwrap()))
            .expect("Tried writing to channel, but there are no args_receivers!");
    }
    
    // drop the sender for receivers to know that there’s nothing more to receive
    drop(args_sender);
    drop(result_sender);

    // init output_vec 
    output_vec.resize_with(input_vec_len, Default::default);
    while let Ok(result_pair) = result_receiver.recv() {
        let (index, f_value) = result_pair;
        output_vec[index] = f_value; 
    }

    // join all worker threads
    for thread in threads {
        thread.join().expect("Panic occurred in thread");
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
