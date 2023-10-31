use anyhow::{bail, Context, Error};
use base64::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use swc_common::{FileName, SourceFile};
use url::Url;

fn read_inline_sourcemap(data_url: Option<&str>) -> Result<Option<sourcemap::SourceMap>, Error> {
    match data_url {
        Some(data_url) => {
            let url = Url::parse(data_url).with_context(|| {
                format!(
                    "failed to parse_program inline source map url\n{}",
                    data_url
                )
            })?;

            let idx = match url.path().find("base64,") {
                Some(v) => v,
                None => {
                    bail!(
                        "failed to parse_program inline source map: not base64: {:?}",
                        url
                    )
                }
            };

            let content = url.path()[idx + "base64,".len()..].trim();
            let res = BASE64_STANDARD
                .decode(content.as_bytes())
                .context("failed to decode base64-encoded source map")?;

            Ok(Some(sourcemap::SourceMap::from_slice(&res).context(
                "failed to read input source map from inlined base64 encoded \
                             string",
            )?))
        }
        None => {
            bail!("failed to parse_program inline source map: `sourceMappingURL` not found")
        }
    }
}

fn read_file_sourcemap(
    data_url: Option<&str>,
    name: &FileName,
) -> Result<Option<sourcemap::SourceMap>, Error> {
    match &name {
        FileName::Real(filename) => {
            let dir = match filename.parent() {
                Some(v) => v,
                None => {
                    bail!("unexpected: root directory is given as a input file")
                }
            };

            let map_path = match data_url {
                Some(data_url) => {
                    let mut map_path = dir.join(data_url);
                    if !map_path.exists() {
                        // Old behavior. This check would prevent
                        // regressions.
                        // Perhaps it shouldn't be supported. Sometimes
                        // developers don't want to expose their source
                        // code.
                        // Map files are for internal troubleshooting
                        // convenience.
                        map_path = PathBuf::from(format!("{}.map", filename.display()));
                        if !map_path.exists() {
                            bail!(
                                "failed to find input source map file {:?} in \
                                             {:?} file",
                                map_path.display(),
                                filename.display()
                            )
                        }
                    }

                    Some(map_path)
                }
                None => {
                    // Old behavior.
                    let map_path = PathBuf::from(format!("{}.map", filename.display()));
                    if map_path.exists() {
                        Some(map_path)
                    } else {
                        None
                    }
                }
            };

            match map_path {
                Some(map_path) => {
                    let path = map_path.display().to_string();
                    let file = File::open(&path)?;

                    Ok(Some(sourcemap::SourceMap::from_reader(file).with_context(
                        || {
                            format!(
                                "failed to read input source map
                            from file at {}",
                                path
                            )
                        },
                    )?))
                }
                None => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

pub fn get_orig_src_map(fm: &SourceFile) -> Result<Option<sourcemap::SourceMap>, Error> {
    let s = "sourceMappingURL=";
    let idx = fm.src.rfind(s);

    let data_url = idx.map(|idx| {
        let data_idx = idx + s.len();
        if let Some(end) = fm.src[data_idx..].find('\n').map(|i| i + data_idx + 1) {
            &fm.src[data_idx..end]
        } else {
            &fm.src[data_idx..]
        }
    });

    Ok(match read_inline_sourcemap(data_url) {
        Ok(r) => r,
        Err(_) => {
            // Load original source map if possible
            match read_file_sourcemap(data_url, &fm.name) {
                Ok(v) => v,
                Err(_) => None,
            }
        }
    })
}
