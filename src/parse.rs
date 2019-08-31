use crate::process::RustcOpts;

use either::Either;
use failure::ResultExt as _;
use maplit::btreemap;
use structopt::StructOpt as _;

use std::collections::BTreeMap;
use std::{iter, mem};

pub(crate) fn cargo_build_vv_stderr_to_opts_and_envs(
    stderr: &str,
) -> crate::Result<Vec<(BTreeMap<String, String>, RustcOpts)>> {
    // https://github.com/rust-lang/cargo/blob/5218d04b3160c62b99f3decbcda77f73d321bf58/src/cargo/util/process_builder.rs#L34-L59
    // https://github.com/sfackler/shell-escape/blob/81621d00297d89c98fb4d5ceb55ad3cd7c1fa69c/src/lib.rs

    use combine::char::{char, string};
    use combine::easy::{self, Info};
    use combine::parser::choice::or;
    use combine::parser::range::recognize;
    use combine::stream::state::{SourcePosition, State};
    use combine::{choice, eof, many, none_of, satisfy, skip_many, skip_many1, Parser};

    type Input<'a> = easy::Stream<State<&'a str, SourcePosition>>;

    #[cfg(windows)]
    fn maybe_escaped<'a>() -> impl Parser<Input = Input<'a>, Output = String> {
        use combine::parser;

        many(or(
            char('"')
                .with(parser(|input| {
                    let mut acc = "".to_owned();
                    let mut num_backslashes = 0;
                    skip_many(satisfy(|c| match c {
                        '\\' => {
                            num_backslashes += 1;
                            true
                        }
                        '"' if num_backslashes % 2 == 1 => {
                            let num_backslashes = mem::replace(&mut num_backslashes, 0);
                            (0..num_backslashes / 2).for_each(|_| acc.push('\\'));
                            acc.push('"');
                            true
                        }
                        '"' => {
                            (0..num_backslashes / 2).for_each(|_| acc.push('\\'));
                            false
                        }
                        c => {
                            let num_backslashes = mem::replace(&mut num_backslashes, 0);
                            (0..num_backslashes).for_each(|_| acc.push('\\'));
                            acc.push(c);
                            true
                        }
                    }))
                    .parse_stream(input)
                    .map(|((), consumed)| (acc, consumed))
                }))
                .skip(char('"')),
            recognize(skip_many1(satisfy(|c| match c {
                '"' | '\t' | '\n' | ' ' => false,
                _ => true,
            })))
            .map(ToOwned::to_owned),
        ))
    }

    #[cfg(unix)]
    fn maybe_escaped<'a>() -> impl Parser<Input = Input<'a>, Output = String> {
        many(choice((
            char('\'')
                .with(recognize(skip_many(none_of("'!".chars()))))
                .skip(char('\'')),
            char('\\').with(or(string("'"), string("!"))).map(|s| s),
            recognize(skip_many1(satisfy(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' | '/' | ',' | '.' | '+' => true,
                _ => false,
            }))),
        )))
    }

    let (mut envs_and_args, mut envs, mut args) = (vec![], btreemap!(), vec![]);

    skip_many(
        skip_many(char(' '))
            .with(choice((
                char('[')
                    .with(skip_many1(none_of("]\n".chars())))
                    .skip(string("] "))
                    .skip(skip_many(none_of(iter::once('\n')))),
                or(
                    char('C').with(or(string("hecking"), string("ompiling"))),
                    char('F').with(or(string("inished"), string("resh"))),
                )
                .with(skip_many1(none_of(iter::once('\n')))),
                string("Running `").with(
                    skip_many(
                        skip_many(char(' '))
                            .with(recognize(skip_many(satisfy(|c| match c {
                                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
                                _ => false,
                            }))))
                            .and(or(
                                char('=').with(maybe_escaped()).map(Either::Left),
                                maybe_escaped().map(Either::Right),
                            ))
                            .and_then(|(fst, rest)| {
                                let rest_is_empty = match &rest {
                                    Either::Left(rest) | Either::Right(rest) => rest.is_empty(),
                                };
                                if fst.is_empty() && rest_is_empty {
                                    envs_and_args.push((
                                        mem::replace(&mut envs, btreemap!()),
                                        mem::replace(&mut args, vec![]),
                                    ));
                                    Err(easy::Error::Expected(Info::Borrowed("`")))
                                } else {
                                    // https://github.com/rust-lang/cargo/blob/5218d04b3160c62b99f3decbcda77f73d321bf58/src/cargo/util/process_builder.rs#L43
                                    match rest {
                                        Either::Left(mut rest) => {
                                            if !fst.is_empty()
                                                && (!cfg!(windows) || rest.ends_with("&&"))
                                                && args.is_empty()
                                            {
                                                if cfg!(windows) {
                                                    rest.pop();
                                                    rest.pop();
                                                }
                                                envs.insert(fst.to_owned(), rest);
                                            } else {
                                                args.push(format!("{}={}", fst, rest));
                                            }
                                            Ok(())
                                        }
                                        Either::Right(rest) => {
                                            if !(cfg!(windows) && rest == "set" && args.is_empty())
                                            {
                                                args.push(format!("{}{}", fst, rest));
                                            }
                                            Ok(())
                                        }
                                    }
                                }
                            }),
                    )
                    .skip(char('`')),
                ),
            )))
            .skip(char('\n')),
    )
    .skip(eof())
    .easy_parse(State::with_positioner(stderr, SourcePosition::new()))
    .map_err(|e| e.map_range(ToOwned::to_owned))
    .with_context(|_| crate::ErrorKind::ParseCargoBuildVvStderr {
        stderr: stderr.to_owned(),
    })?;

    envs_and_args
        .into_iter()
        .filter(|(_, args)| args.len() > 1) // build-script-build
        .map(|(envs, args)| {
            let opts = RustcOpts::from_iter_safe(&args)
                .with_context(|_| crate::ErrorKind::ParseRustcOptions { args })?;
            Ok((envs, opts))
        })
        .collect()
}
