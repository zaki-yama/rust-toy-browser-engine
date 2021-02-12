pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod painting;
pub mod style;

fn main() {
    let value = Some(String::from("hello"));

    let result = value.iter().any(|name| {
        println!("{}", name);
        name.eq("hello")
    });
    println!("{}", result);
}
