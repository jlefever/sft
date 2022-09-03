mod index_map;
mod prelude;
mod preprocessing;

use prelude::*;
use preprocessing::*;

// type ItemId = usize;

// struct Dep(ItemId, ItemId);

fn main() {
    let pairs = vec![(0, 1), (1, 1), (0, 3)];
    let adj = adjmat(pairs);

    let adj = array![[0, 0, 1], [1, 0, 1], [1, 1, 0]];

    println!("ADJ:\n{}", adj);

    println!("SIM (ROWS):\n{}", simmat(adj.view(), ROWS, jaccard));
    println!("SIM (COLS):\n{}", simmat(adj.view(), COLS, jaccard));

    // let axis = Axis(0);

    // for (a, b) in iproduct!(adj.axis_iter(axis), adj.axis_iter(axis)) {
    //     let res = jaccard((a, b));
    //     println!("{} (-) {}: {}", a, b, res);
    // }

    // let x = Array1::from_iter(iproduct!(adj.axis_iter(axis),
    // adj.axis_iter(axis)).map(jaccard)); let y = x.into_shape(adj.
    // shape()).unwrap(); println!("{}", y);

    // let out = Array1::from(iproduct!(adj.axis_iter(axis),
    // adj.axis_iter(axis).map(jaccard)));

    // println!("ADJ:\n{}", adj);
    // for x in adj.axis_iter(Axis(0)) {
    //     println!("{}", x);
    // }

    // let x = Array1::from(vec![1, 0, 1]);
    // let y = Array1::from(vec![1, 1, 1]);
    // let z = jaccard(x.view(), y.view());
    // println!("{}", z);

    // println!("Input:\n{}", adj);
    // adj.map_axis(ROWS, |x| {
    //     println!("Intermediate:\t{}", x);
    //     x
    // });
    // println!("Output:\n{}", adj);
}

// fn main() {
//     let foo = Rc::new("Foo.java".to_owned());
//     let bar = Rc::new("Bar.java".to_owned());
//     let baz = Rc::new("Baz.java".to_owned());

//     let mut index_map = IndexMap::new();
//     index_map.put(foo.clone());
//     index_map.put(bar.clone());
//     index_map.put(baz.clone());

//     let res = index_map.map(vec![&foo.clone(), &bar.clone(),
// &baz.clone()]).collect_vec();

//     println!("{:?}", res);
// }
