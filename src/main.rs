use pekoai::*;
use ppekom::*;

fn main() {
    to_ppm("test_images/edgecase.qoi");
    to_ppm("test_images/qoi_logo.qoi");
    to_ppm("test_images/testcard.qoi");
    to_ppm("test_images/testcard_rgba.qoi");
}

fn to_ppm(path: &str) {
    let image = load_qoi(path).unwrap();
    let mut new_name = path.to_string();
    new_name.push_str("written.ppm");
    println!("{}", new_name);
    write_ppm(new_name, &image).unwrap();
}
