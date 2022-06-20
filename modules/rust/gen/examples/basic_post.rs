use treeldr_rust_macros::tldr;

#[tldr(
	"examples/xsd.tldr",
	"examples/schema.org.tldr",
	"modules/rust/gen/examples/vc.tldr",
	"modules/rust/gen/examples/rebase.tldr",
	"modules/rust/gen/examples/basic_post.tldr"
)]
mod schema {
	#[prefix("https://www.w3.org/2018/credentials#")]
	mod vc {}

	#[prefix("https://example.com/rebase/")]
	mod rebase {}

	#[prefix("https://example.com/example/")]
	mod basic_post {}
}

fn main() {
	println!("Hello World!")
}
