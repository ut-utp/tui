//! TODO!

use lc3_assembler::{
    assembler::assemble,
    error::extract_file_errors,
    lexer::Lexer,
    parser::{parse, LeniencyLevel},
};
use lc3_isa::{ADDR_SPACE_SIZE_IN_WORDS, ADDR_SPACE_SIZE_IN_BYTES, util::MemoryDump};
#[cfg(not(target_arch = "wasm32"))]
use lc3_shims::memory::FileBackedMemoryShim;
use lc3_traits::control::metadata::{
    LongIdentifier, ProgramMetadata,
};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Snippet, Annotation, Slice, AnnotationType, SourceAnnotation},
};
use bytes::Buf;
use chrono::{DateTime, Utc};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{blocking, header};

use std::fmt::{self, Display, Result as FmtResult};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, Duration};

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

impl FromStr for ProgramSource {
    type Err = &'static str;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        use ProgramSource::*;

        match src {
            _ if src.len() == 0 => Err("Empty program source!"),
            bad if (
                bad.starts_with("mem:") || bad.starts_with("asm:") || bad.starts_with("imm:")
            ) && bad.len() == 4 => {
                Err("Missing URL!")
            }
            mem if mem.starts_with("mem:") => Ok(MemoryDumpUrl(mem.trim_start_matches("mem:").to_string())),
            asm if asm.starts_with("asm:") => Ok(AssemblyUrl(asm.trim_start_matches("asm:").to_string())),
            imm if imm.starts_with("imm:") => Ok(ImmediateSource(imm.trim_start_matches("imm:").to_string())),

            #[cfg(not(target_arch = "wasm32"))]
            path => Ok(FilePath(PathBuf::from(path.to_string()))),
            #[cfg(target_arch = "wasm32")]
            _ => Err("Program Source must be an immediate, mem URL, or asm URL on wasm."),
        }
    }
}

pub(in crate) fn file_requires_assembly(path: &PathBuf) -> bool {
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

    assemble_mem_dump_str(src, Some(path_str.as_str()), with_os)
}

pub(in crate) fn assemble_mem_dump_str(src: &str, path: Option<&str>, with_os: bool) -> Result<MemoryDump, String> {
    let lexer = Lexer::new(src);
    let cst = parse(lexer, LeniencyLevel::Lenient);

    let errors = extract_file_errors(cst.clone());
    if errors.len() > 0 {
        let mut error_string = String::new();
        for error in errors {
            let label_string = error.message();
            let label = label_string.as_str();
            let annotations = error.annotations();
            let slices = slices(annotations, src, path);
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

// A bad hack..
//
// I think this is the approach we'll have to go with but we should do things to make
// it less bad like having this function return actual errors. (TODO)
//
// Maybe WebWorkers will let us basically have threads which'd be a way to let us
// make requests when the load button is pressed.
//
// Other than that I don't think there's a way (we can't/won't make everything async).
//
// Update: I think Atomics + WebWorkers might let us do this.
#[cfg(target_arch = "wasm32")]
impl ProgramSource {
    pub async fn normalize(&mut self) -> Result<(), ()> {
        use ProgramSource::*;

        match self {
            ImmediateSource(_) => Ok(()),
            AssemblyUrl(asm) => {
                let imm = reqwest::get(asm.as_str()).await.unwrap().text().await.unwrap();
                *self = ImmediateSource(imm);
                Ok(())
            },
            MemoryDumpUrl(_) => Err(()),
        }
    }
}

impl ProgramSource {
    pub(in crate) fn requires_assembly(&self) -> bool {
        use ProgramSource::*;
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            FilePath(p) => file_requires_assembly(p),
            ImmediateSource(_) => true,
            MemoryDumpUrl(_) => false,
            AssemblyUrl(_) => true,
        }
    }

    pub(in crate) fn long_ident(&self) -> LongIdentifier {
        use ProgramSource::*;

        macro_rules! def {
            ($lit:literal) => {
                LongIdentifier::new_from_str_that_crashes_on_invalid_inputs($lit)
            };
        }

        let (default, string): (LongIdentifier, Option<String>) = match self {
            #[cfg(not(target_arch = "wasm32"))]
            FilePath(p) => {
                let file_name = p.file_name()
                    .and_then(|f| f.to_str())
                    .map(|s| s.to_string());

                (def!("<<file>>"), file_name)
            },
            ImmediateSource(_) => (def!("<string>"), None),
            MemoryDumpUrl(url) => (def!("<mem://>"), Some(format!("m:{}", url))),
            AssemblyUrl(url) => (def!("<asm://>"), Some(format!("a:{}", url))),
        };

        let ident: Option<String> = string.map(|s| s.chars()
            .chain(core::iter::repeat(' '))
            .take(LongIdentifier::MAX_LEN)
            .collect());

        ident.and_then(|s| LongIdentifier::new_from_str(s.as_str()).ok())
            .unwrap_or(default)
    }

    pub(in crate) fn to_memory_dump(
        &self,
        with_os: bool,
    ) -> Result<(MemoryDump, ProgramMetadata), String> {
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
                        .map_err(|e| format!("Failed to load `{}` as a MemoryDump; got: {:?}", path.display(), e))?
                        .into()
                };

                let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());

                (mem_dump, last_modified)
            },

            ImmediateSource(src) => {
                (assemble_mem_dump_str(src, None, with_os)?, None)
            },

            #[cfg(not(target_arch = "wasm32"))]
            MemoryDumpUrl(url) | AssemblyUrl(url) => {
                let resp = blocking::get(url)
                    .map_err(|err| format!("Failed to get `{}`: {}", url, err))?;

                let last_modified = resp
                    .headers()
                    .get(header::LAST_MODIFIED)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| DateTime::parse_from_rfc2822(s).ok())
                    .map(|dt| DateTime::from_utc(dt.naive_utc(), Utc))
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
                            memory[idx] = bytes.get_u16_le();
                        }

                        memory.into()
                    },

                    AssemblyUrl(_) => {
                        assemble_mem_dump_str(&resp.text().map_err(|err| {
                            format!(
                                "Error while reading program from `{}`: {}",
                                url,
                                err,
                            )
                        })?, None, with_os)?
                    },

                    _ => unreachable!(),
                };

                (mem_dump, last_modified)
            }

            #[cfg(target_arch = "wasm32")]
            _ => return Err(format!("Call `normalize` on your ProgramSource when on WASM, please.")),
        };

        // TODO: fix this on wasm! (WASM-TIME-FIX)
        #[cfg(not(target_arch = "wasm32"))]
        let mut metadata = ProgramMetadata::new_modified_now(self.long_ident(), &memory_dump);

        #[cfg(target_arch = "wasm32")]
        let mut metadata = ProgramMetadata::new(self.long_ident(), &memory_dump, Duration::from_secs(1));

        if let Some(lm) = override_last_modified {
            metadata.modified_on(lm);
        }

        Ok((memory_dump, metadata))
    }
}
