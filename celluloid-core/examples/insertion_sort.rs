//! cargo run -p celluloid-core --example insertion_sort
//! then: celluloid insertion_sort.json

use celluloid_core::Recorder;

fn main() {
    let mut values: Vec<i32> = (0..30).map(|i| (i * 17 + 5) % 31).collect();

    let mut rec = Recorder::new("Insertion sort");
    rec.frame_delay(80.0);
    let a = rec.array("values", values.iter().copied());
    rec.state(a, "sorted", "#59a14f");
    rec.state(a, "key", "#f28e2b");
    rec.set(a, 0, "sorted");
    rec.frame("start");

    let mut comparisons = 0;
    for i in 1..values.len() {
        rec.set(a, i, "key");
        rec.marker(a, "i", i);
        rec.frame(format!("insert {}", values[i]));

        let key = values[i];
        let mut j = i;
        while j > 0 && values[j - 1] > key {
            j -= 1;
            rec.marker(a, "j", j);
            comparisons += 1;
            rec.var("comparisons", comparisons);
            rec.frame(format!("{} > {key}", values[j]));
        }
        let item = values.remove(i);
        values.insert(j, item);
        rec.move_item(a, i, j);
        rec.set(a, j, "sorted");
        rec.hide_marker(a, "j");
        rec.frame(format!("placed {key} at {j}"));
    }
    rec.hide_marker(a, "i");
    rec.bookmark("sorted");
    rec.frame("sorted");

    rec.save("insertion_sort.json").unwrap();
    println!("wrote insertion_sort.json");
}
