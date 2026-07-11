use blueos_catalog::FeatureCatalog;

fn main() {
    let features = FeatureCatalog::bootstrap();
    let view = features.aggregate_view();

    for group in &view {
        println!("== {} ({}) ==", group.aggregate, group.features.len());
        for id in &group.features {
            println!("  {}", id.0);
        }
        println!();
    }

    println!(
        "total features: {}, aggregates: {}",
        features.features().len(),
        view.len()
    );
}
