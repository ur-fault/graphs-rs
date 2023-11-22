use graph::create_graph;

mod graph;
mod cycle;
mod parsing;

fn main() {
    let graph = create_graph! {
        1 => 2;
    };

    println!("{:?}", graph);
}
