use std::{
    fs::File,
    io::{self, Read},
};

pub fn run_cat_command(paths: Vec<String>) {
    //REMEMBER: the first element in the vector is the command it self

    //remove the single quote from the paths
    let x = paths
        .iter()
        .map(|path| {
            if path.starts_with('\'') && path.ends_with('\'') {
                path[1..path.len() - 1].to_string()
            } else {
                path.clone()
            }
        })
        .collect::<Vec<String>>();

    let mut total_content: Vec<String> = vec![];

    for (index, path) in x.iter().enumerate() {
        // pass the command - the first element in the vector
        if index == 0 {
            continue;
        }

        let file_content = read_file(path);
        let content_str = match file_content {
            Result::Ok(content) => content,
            Result::Err(error) => {
                eprintln!(
                    "Error opening file, ERROR: {}, FOR PATH GIVEN: {}",
                    error, path
                );
                return;
            }
        };
        total_content.push(content_str);
    }

    println!("{}", total_content.join(" "));
}

fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = match File::open(path.trim()) {
        Result::Ok(file) => file,
        Result::Err(error) => return Result::Err(error),
    };

    let mut file_contents = String::new();
    let read_operation = file.read_to_string(&mut file_contents);

    if let Result::Err(error) = read_operation {
        return Result::Err(error);
    }

    Result::Ok(file_contents)
}
