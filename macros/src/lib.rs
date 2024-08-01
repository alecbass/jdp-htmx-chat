use std::fs::read_to_string;
use std::path::PathBuf;

use quote::quote;

use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn load_query(input: TokenStream) -> TokenStream {
    const QUERY_LOCATION: &'static str = "database/queries";

    if input.is_empty() {
        panic!("No query file provided");
    }

    let tokens = input
        .into_iter()
        // An arugment of "query.sql" will result in a string which includes quotes
        .map(|token| token.to_string().replace("\"", ""))
        .collect::<Vec<_>>();

    if tokens.len() > 1 {
        panic!("Only one query file can be passed");
    }

    let input = tokens
        .into_iter()
        .map(|word| word.to_string())
        .collect::<String>();

    // let input = tokens.get(0);

    if input.is_empty() {
        panic!("No query file provided");
    }

    // Get the path i.e. database/queries
    let path = PathBuf::from(QUERY_LOCATION);

    // Get the file name i.e. query.sql
    let file_name = PathBuf::from(input);

    // Get the full path i.e. database/queries/query.sql
    let file_path = path.join(&file_name);

    let does_file_exist = file_path.exists();

    if !does_file_exist {
        // No file to read an SQL query from

        let file_path_str = file_path.to_str();

        if let Some(file_path_str) = file_path_str {
            panic!("Query file does not exist: {}", file_path_str);
        } else {
            panic!("Query file does not exist");
        }
    }

    let extension = file_path.extension();

    if extension.is_none() {
        // Be sure we're reading an SQL file
        panic!("Query file requires an .sql extension");
    }

    let extension = extension.unwrap();

    if extension != "sql" {
        // Be sure we're reading an SQL file
        panic!("File is not an SQL file");
    }

    let query = read_to_string(file_path).unwrap();

    TokenStream::from(quote! {
        #query
    })
}
