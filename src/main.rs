use pekoai::*;

fn main() {
    testing("edgecase.qoi");
    testing("qoi_logo.qoi");
    testing("testcard.qoi");
    testing("testcard_rgba.qoi");
}

// already first tested the load_qoi function with write_ppm from the ppekom library.
fn testing(name: &str) {
    let mut path = "test_images/".to_string();
    path.push_str(name);
    let original = load_qoi(path.as_str()).unwrap();
    let mut written_path = "test_images/written_".to_string();
    written_path.push_str(name);
    write_qoi(written_path.as_str(), &original).unwrap();
    let written = load_qoi(written_path.as_str()).unwrap();
    if original.width() != written.width() {
        println!("Expected width: {}", original.width());
        println!("Actual width: {}", written.width());
    }
    if original.height() != written.height() {
        println!("Expected height: {}", original.height());
        println!("Actual height: {}", written.height());
    }
    let mut diff_pixels_count = 0;
    for x in 0..original.width() {
        for y in 0..original.height() {
            if original.get(x, y).unwrap() != written.get(x, y).unwrap() {
                println!("Expected pixel: {}", original.get(x, y).unwrap());
                println!("Actual pixel: {}", written.get(x, y).unwrap());
                diff_pixels_count += 1;
            }
        }
    }
    println!(
        "Completed test with {} pixels different from expected.",
        diff_pixels_count
    );
}
