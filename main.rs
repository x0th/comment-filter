// @Todo:
// better error handling
// list of ignored files?

use std::{fs, io, env};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::ffi::OsStr;

#[derive(Debug)]
struct CommentInfo {
	filename: String,
	comments: HashMap<String, Vec<String>>
}

#[derive(Debug)]
struct Config {
	spec_symbols: Vec<char>,
	spec_words: Option<Vec<String>>,
	ignored_words: Option<Vec<String>>,
}

impl Config {
	fn default() -> Config {
		Config { spec_symbols: vec!['@'], spec_words: None, ignored_words: None }
	}
}

fn print_help() {
	println!("\nUsage: ./comment-filter directory_or_file [OPTIONS...]\n\n");
	println!(" --help, -h\t\t\t\tShow help message.\n");
	println!(" --depth, -d *int*\t\t\tIf directory is selected, set the depth of reqursion. Default: full reqursion.\n\t\t\t\t\tExample: -d 3");
	println!(" --config *file*\t\t\tSelect file with parsing configuration.\n\t\t\t\t\tExample: --config comment-filter.conf\n");
	println!(" --symbols, -s *string*\t\t\tSelect special symbol markers for names of categories. Default: @.\n\t\t\t\t\tExample: -s @*+-=");
	println!(" --categories, -c [*string*,...]\tSelect special category names. Default: all words.\n\t\t\t\t\tExample: -c [Name,Name2]");
	println!(" --ignore, -i [*string*,...]\t\tSelect names of categories to ignore. Default: none.\n\t\t\t\t\tExample: -i [Ignore,This]");
}

fn config_from_file(path: &str) -> Config {
	let mut config = Config::default();
	let mut f = File::open(path).unwrap();
	let mut content = String::new();
	f.read_to_string(&mut content).unwrap();
	let lines = content.lines().collect::<Vec<&str>>();

	let mut i = 0;
	while i < lines.len() {
		if lines[i].contains("[") &&  lines[i].contains("]") {
			match &lines[i].trim() {
				&"[Symbols]" => {
					if i+1 != lines.len() && &lines[i+1].trim() != &"" {
						i += 1;
						config.spec_symbols = lines[i].chars().collect();
					}
				},
				&"[Categories]" => {
					let mut spec_words = Vec::new();
					i += 1;
					while i < lines.len() && &lines[i].trim() != &"" {
						spec_words.push(String::from(lines[i]));

						i += 1;
					}
					if !spec_words.is_empty() { config.spec_words = Some(spec_words); }
				},
				&"[Ignore]" => {
					let mut ignored_words = Vec::new();
					i += 1;
					while i < lines.len() && &lines[i].trim() != &"" {
						ignored_words.push(String::from(lines[i]));

						i += 1;
					}
					if !ignored_words.is_empty() { config.ignored_words = Some(ignored_words); }
				},
				_ => ()
			}
		}

		i += 1;
	}
	config
}

fn save_to_file(comment_info: Vec<CommentInfo>) {
	let mut f = match File::create("filtered_comments.txt") {
		Err(error) => panic!("{:?}", error),
		Ok(f)      => f,
	};

	for info in comment_info {
		if !info.comments.is_empty() {
			let mut s = info.filename;
			s += "\n\n";
			
			for (key, value) in &info.comments {
				s += "\t";
				s += key;
				s += "\n\n";
				for v in value { s += v; s+= "\n"; }
				s += "\n"
			}

			match f.write_all(s.as_bytes()) {
				Err(error) => panic!("{:?}", error),
				Ok(_)      => continue,
			}
		}
	}
	println!("\nSuccesfully wrote filtered comments to file filtered_comments.txt!");
}

fn check_word_against_config(config: &Config, s: &String) -> bool {
	match &config.spec_words {
		Some(v) =>  {
			match &config.ignored_words {
				Some(ign) => {
					if v.contains(&String::from(&s[1..])) && !ign.contains(&String::from(&String::from(&s[1..]))) {
						return true
					}
				},
				None      => {
					if v.contains(&String::from(&s[1..])) {
						return true
					}
				}
			}
		},
		None    => {
			match &config.ignored_words {
				Some(ign) => {
					if !ign.contains(&String::from(&String::from(&s[1..]))) {
						return true
					}
				},
				None      => {
					return true
				}
			}
		}
	}
	false
}

fn parse_comment_block(lines: &Vec<&str>, start_index: usize, file_info: &mut CommentInfo, config: &Config) -> usize {
	let mut i                     = start_index;
	let mut curr_cat: Vec<String> = Vec::new();
	while i < lines.len() {
		for sym in &config.spec_symbols {
			if lines[i].contains(&sym.to_string()) {
				let mut found = false;
				let mut s = String::from("");
				for c in lines[i].chars() {
					if c == *sym                            { found = true; }
					else if found && (c == ':' || c == ' ') { break; }

					if found { s.push(c); }
				}
				if !curr_cat.contains(&s) && check_word_against_config(config, &s) {
					curr_cat.push(s);
				}
			}
		}

		if lines[i].contains("*/") || i == lines.len() - 1 {
			let mut s = String::from("");
			for j in start_index..i + 1 {
				s += &format!("\t\t{} {}\n", j+1, lines[j]);
			}

			for cat in &curr_cat {
				if !file_info.comments.contains_key(&cat.to_string()) {
					file_info.comments.insert(cat.to_string(), Vec::new());
				}
				let v = file_info.comments.get_mut(&cat.to_string()).unwrap();
				v.push(s.to_string());
			}

			return i;
		}

		i += 1;
	}
	i
}

fn parse_comments(lines: &Vec<&str>, start_index: usize, file_info: &mut CommentInfo, is_slash_slash: bool, config: &Config) -> usize {
	let mut i                     = start_index;
	let mut curr_cat: Vec<String> = Vec::new();
	let symb: &str;
	if is_slash_slash { symb = "//"; } else { symb = "#"; }
	while i < lines.len() {
		for sym in &config.spec_symbols {
			if lines[i].contains(&sym.to_string()) {
				let mut found = false;
				let mut s = String::from("");
				for c in lines[i].chars() {
					if c == *sym                            { found = true; }
					else if found && (c == ':' || c == ' ') { break; }

					if found { s.push(c); }
				}
				if !curr_cat.contains(&s) && check_word_against_config(config, &s) {
					curr_cat.push(s);
				}
			}
		}

		if !lines[i].contains(symb) || i == lines.len() - 1 {
			if i == lines.len() - 1 { i += 1; }
			let mut s = String::from("");
			for j in start_index..i {
				s += &format!("\t\t{} {}\n", j+1, lines[j]);
			}

			for cat in &curr_cat {
				if !file_info.comments.contains_key(&cat.to_string()) {
					file_info.comments.insert(cat.to_string(), Vec::new());
				}
				let v = file_info.comments.get_mut(&cat.to_string()).unwrap();
				v.push(s.to_string());
			}

			return i;
		}

		i += 1;
	}
	i
}

fn add_file(path: &Path, comment_info: &mut Vec<CommentInfo>, config: &Config) -> io::Result<()> {
	let mut file_info = CommentInfo { filename: String::from(path.to_str().unwrap()), comments: HashMap::new() };

	let mut f = File::open(path)?;
	let mut content = String::new();
	match f.read_to_string(&mut content) {
		Err(_err) => {
			return Ok(())
		},
		_         => ()
	}
	let lines = content.lines().collect::<Vec<&str>>();
	
	let mut i = 0;
	while i < lines.len() {
		if lines[i].contains("/*")      { i = parse_comment_block(&lines, i, &mut file_info, config); }
		else if lines[i].contains("//") { i = parse_comments(&lines, i, &mut file_info, true, config); }
		else if lines[i].contains("#")  { i = parse_comments(&lines, i, &mut file_info, false, config); }

		i += 1;
	}

	comment_info.push(file_info);

	Ok(())
}

fn visit_dir(dir: &Path, comment_info: &mut Vec<CommentInfo>, counter: i32, rec_depth: i32, config: &Config) -> io::Result<()> {
	if rec_depth != -1 && counter == rec_depth { return Ok(()) }

	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let entry_path = entry?.path();
			visit_dir(&entry_path, comment_info, counter + 1, rec_depth, config)?;
		}
	} else {
		if dir.file_name() != Some(OsStr::new("filtered_comments.txt")) {
			add_file(dir, comment_info, config)?;
		}
	}
	Ok(())
}

fn parse_file_or_dir(file_or_dir: &String, rec_depth: i32, config: &Config) -> Option<Vec<CommentInfo>> {
	let mut comment_info: Vec<CommentInfo> = Vec::new();

	visit_dir(Path::new(file_or_dir), &mut comment_info, 0, rec_depth, config).unwrap();

	Some(comment_info)
}


fn parse_arguments(args: Vec<String>) {
	let mut config       = Config::default();
	let mut comment_info = None;
	let mut file_or_dir  = None;
	let mut depth        = -1;

	let args_len   = args.len();
	let mut cursor = 1;

	if args_len < 2 { panic!("Not enough arguments! Try --help option for list of commands"); }

	if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) { print_help(); return; }

	while cursor < args_len {
		if cursor == 1 {
			if args[cursor].chars().next().unwrap() == '-' { panic!("First argument is not a file or directory!"); }

			file_or_dir = Some(&args[cursor]);

			cursor += 1;
		} else if &args[cursor] == "--depth" || &args[cursor] == "-d" {
			let d_r = args[cursor+1].parse();
			if !d_r.is_ok() { panic!("Entered depth in NaN"); };
			depth = d_r.unwrap();
			if depth != -1 { depth+=1 };

			cursor += 2;
		} else if &args[cursor] == "--symbols" || &args[cursor] == "-s" {
			let spec_sym: Result<String, _> = args[cursor+1].parse();
			config.spec_symbols.clear();
			for c in spec_sym.unwrap().chars() {
				config.spec_symbols.push(c);
			}

			cursor += 2;
		} else if &args[cursor] == "--categories" || &args[cursor] == "-c" {
			let spec_words: String = args[cursor+1].parse().unwrap();
			if !spec_words.contains("[") || !spec_words.contains("]") { panic!("Wrong formatting on list of names of categories."); }
			let mut spec_words: Vec<&str> = spec_words.split(",").collect();
			
			spec_words[0] = &spec_words[0][1..];
			let l = spec_words.len()-1;
			spec_words[l] = &spec_words[l][..spec_words[l].len()-1];
			
			if spec_words != vec![""] {
				config.spec_words = Some(spec_words.iter().map(|s| String::from(*s)).collect());
			}

			cursor += 2;
		} else if &args[cursor] == "--ignore" || &args[cursor] == "-i" {
			let ignored_words: String = args[cursor+1].parse().unwrap();
			if !ignored_words.contains("[") || !ignored_words.contains("]") { panic!("Wrong formatting on list of ignored words."); }
			let mut ignored_words: Vec<&str> = ignored_words.split(",").collect();
			
			ignored_words[0] = &ignored_words[0][1..];
			let l = ignored_words.len()-1;
			ignored_words[l] = &ignored_words[l][..ignored_words[l].len()-1];
			
			if ignored_words != vec![""] {
				config.ignored_words = Some(ignored_words.iter().map(|s| String::from(*s)).collect());
			}

			cursor += 2;
		} else if &args[cursor] == "--config" {
			 config = config_from_file(&args[cursor+1]);
			 break;
		} else {
			panic!("Problem parsing console arguments. Check if entered arguments are in correct format.");
		}
	}	

	if file_or_dir != None {
		println!("Beginning to parse file/directory with given parameters.");
		comment_info = parse_file_or_dir(file_or_dir.unwrap(), depth, &config);
	}

	match comment_info {
		Some(x) => save_to_file(x),
		None    => panic!("Something went wrong!"),
	}
}

fn main() {
    parse_arguments(env::args().collect());
}
