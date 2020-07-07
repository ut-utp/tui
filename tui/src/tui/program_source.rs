//! TODO!

use lc3_isa::{ADDR_SPACE_SIZE_IN_WORDS, ADDR_SPACE_SIZE_IN_BYTES};

use byteorder::{ReadBytesExt, LittleEndian};
use chrono::DateTime;
use reqwest::{blocking, header};

use std::path::PathBuf;
use std::fmt::{self, Display, Result as FmtResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramSource {
    // No files on wasm.
    //
    // TODO: eventually we can use web_sys APIs to have it so that File on wasm means
    // a local file.
    #[cfg(not(target_arch = "wasm32"))]
    FilePath(PathBuf),

    ImmediateSource(String),

    MemoryDumpUrl(String),
    AssemblyUrl(String),
}

impl Display for ProgramSource {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> FmtResult {
        use ProgramSource::*;
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            FilePath(f) => {
                write!(fmt, "{}", f.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("<unprintable>"))
            },
            ImmediateSource(_) => write!(fmt, "<immediate>"),
            MemoryDumpUrl(url) => write!(fmt, "mem:{}", url),
            AssemblyUrl(url) => write!(fmt, "asm:{}", url),
        }
    }
}

// TODO: impl FromStr?
// or: have different flags produce different variants

pub fn file_requires_assembly(path: &PathBuf) -> bool {
    return match path.extension() {
        Some(ext) => ext == "asm",
        _ => false,
    }
}

pub(in crate) fn create_snippet<'input>(label: &'input str, slices: Vec<Slice<'input>>) -> Snippet<'input> {
    Snippet {
        title: Some(Annotation {
            label: Some(label),
            id: None,
            annotation_type: AnnotationType::Error
        }),
        footer: vec![],
        slices,
        opt: FormatOptions { color: true, anonymized_line_numbers: false }
    }
}

pub(in crate) fn slices<'input>(annotations: Vec<SourceAnnotation<'input>>, source: &'input str, origin: Option<&'input str>) -> Vec<Slice<'input>> {
    let mut slices = Vec::new();
    if !annotations.is_empty() {
        slices.push(
            Slice {
                source,
                origin,
                line_start: 1,
                fold: true,
                annotations,
            }
        );
    }
    slices
}

pub(in crate) fn assemble_mem_dump(path: &PathBuf, with_os: bool) -> Result<MemoryDump, String> {
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let string = fs::read_to_string(path).unwrap();
    let src = string.as_str();

    assemble_mem_dump_str(src, with_os)
}

pub(in crate) fn assemble_mem_dump_str(src: &str, with_os: bool) -> Result<MemoryDump, String> {
    let lexer = Lexer::new(src);
    let cst = parse(lexer, LeniencyLevel::Lenient);

    let errors = extract_file_errors(cst.clone());
    if errors.len() > 0 {
        let mut error_string = String::new();
        for error in errors {
            let label_string = error.message();
            let label = label_string.as_str();
            let annotations = error.annotations();
            let slices = slices(annotations, src, Some(path_str.as_str()));
            let snippet = create_snippet(label, slices);
            let dl = DisplayList::from(snippet);
            error_string = format!("{}\n{}", error_string, dl);
        }
        let error_string = error_string.replace("\n", "\n|");
        return Err(error_string);
    }

    let background = if with_os {
        Some(lc3_os::OS_IMAGE.clone())
    } else {
        None
    };

    Ok(assemble(cst.objects, background))  // TODO: can still fail. fix in assembler.
}

impl ProgramSource {
    pub fn requires_assembly(&self) -> bool {
        use ProgramSource::*;
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            FilePath(p) => file_requires_assembly(p),
            ImmediateSource(_) => true,
            MemoryDumpUrl(_) => false,
            AssemblyUrl(_) => true,
        }
    }

    pub fn long_ident(&self) -> LongIdentifier {
        use ProgramSource::*;

        macro_rules! def {
            ($lit:literal) => {
                LongIdentifier::new_from_str_that_crashes_on_invalid_inputs($lit)
            };
        }

        let (default, string): (LongIdentifer, Option<String>) = match self {
            #[cfg(not(target_arch = "wasm32"))]
            FilePath(p) => {
                let file_name = p.file_name()
                    .and_then(|f| f.to_str())
                    .and_then(ToString::to_string);

                (def!("<<file>>"), file_name)
            },
            ImmediateSource(_) => (def!("<string>"), None),
            MemoryDumpUrl(url) => (def!("<mem://>"), Some(format!("m:{}", url))),
            AssemblyUrl(url) => (def!("<asm://>"), Some(format!("a:{}", url))),
        };

        let ident: String = string.chars()
            .chain(core::iter::repeat(' '))
            .take(LongIdentifier::MAX_LEN)
            .collect();

        LongIdentifer::new_from_str(ident.as_str())
            .unwrap_or(default)
    }

    pub fn to_memory_dump(
        &self,
        with_os: bool,
    ) -> Result<(Box<MemoryDump>, ProgramMetadata), String> {
        use ProgramSource::*;

        let (memory_dump, override_last_modified) = match self {
            #[cfg(not(target_arch = "wasm32"))]
            FilePath(path) => {
                if !path.exists() {
                    return Err(format!("`{}` does not exist!", path.display()))
                }

                let mem_dump = if file_requires_assembly(path) {
                    assemble_mem_dump(path, with_os)?
                } else {
                    FileBackedMemoryShim::from_existing_file(path)
                        .map_err(|e| format!("Failed to load `{}` as a MemoryDump; got: {:?}", p, e))?
                        .into()
                };

                let last_modified = path.metadata().and_then(|m| m.modified());

                (mem_dump, last_modified)
            },

            ImmediateSource(src) => {
                (assemble_mem_dump_str(src)?, None)
            },

            MemoryDumpUrl(url) | AssemblyUrl(url) => {
                let resp = reqwest::blocking::get(url)
                    .map_err(|err| format!("Failed to get `{}`: {}", url, err))?;

                let last_modified = resp
                    .headers()
                    .get(reqwest::header::LAST_MODIFIED)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| DateTime::parse_from_rfc2822(s).ok().into())
                    // TODO: use the local time zone instead?
                    .and_then(|dt: DateTime<Utc>| {
                        let nanos = dt.timestamp_nanos();
                        let nanos = if nanos.is_negative() { return None; } else {
                            nanos as u64
                        };

                        SystemTime::UNIX_EPOCH.checked_add(Duration::from_nanos(nanos))
                    });

                let mem_dump = match self {
                    MemoryDumpUrl(_) => {
                        let mut bytes = resp.bytes().map_err(|err|
                            format!("Couldn't get bytes from `{}`: {}", url, err))?;

                        // Check that len is correct.
                        // Use the MemoryDump::from_read thing
                        // todo!()

                        if bytes.len() != ADDR_SPACE_SIZE_IN_BYTES {
                            return Err(format!(
                                "MemoryDump from `{}` is the wrong size: expected {} bytes but got {} bytes",
                                url,
                                ADDR_SPACE_SIZE_IN_BYTES,
                                bytes.len()
                            ))
                        }

                        let mut memory = [0u16; ADDR_SPACE_SIZE_IN_WORDS];
                        for idx in 0..ADDR_SPACE_SIZE_IN_WORDS {
                            memory[idx] = bytes.read_u16::<LittleEndian>()
                                .map_err(|err| format!(
                                    "Error while reading MemoryDump from `{}` on word {:#4X}: {}",
                                    url,
                                    idx,
                                    err,
                                ))?;
                        }

                        memory.into()
                    },

                    AssemblyUrl(_) => {
                        assemble_mem_dump_str(resp.text().map_err(|err| {
                            format!(
                                "Error while reading program from `{}`: {}",
                                url,
                                err,
                            )
                        })?)?
                    },

                    _ => unreachable!(),
                };

                (mem_dump, last_modified)
            }
        };

        // TODO: fix this on wasm!
        let mut metadata = ProgramMetadata::new_modified_now(self.long_ident(), &memory_dump);

        if let Some(lm) = override_last_modified {
            metadata.modified_on(t);
        }
    }
}
