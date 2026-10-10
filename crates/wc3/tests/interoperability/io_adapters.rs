use std::error::Error as _;
use std::io::{self, Read as IoRead, Write as IoWrite};
use wc3::model::mdl::Write as _;
use wc3::model::mdx::Write as _;
use wc3::model::IoError;
use wc3::model::{mdl, mdx, DynamicModel, Model, V800};

struct ShortIo {
    bytes: Vec<u8>,
    interrupted: bool,
}
impl IoRead for ShortIo {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let len = buffer.len().min(self.bytes.len()).min(3);
        buffer[..len].copy_from_slice(&self.bytes[..len]);
        self.bytes.drain(..len);
        Ok(len)
    }
}
impl IoWrite for ShortIo {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let len = bytes.len().min(3);
        self.bytes.extend_from_slice(&bytes[..len]);
        Ok(len)
    }
    fn flush(&mut self) -> io::Result<()> {
        panic!("adapters must not flush")
    }
}
struct Failing;
impl IoRead for Failing {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "source failure",
        ))
    }
}
impl IoWrite for Failing {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "sink failure"))
    }
    fn flush(&mut self) -> io::Result<()> {
        panic!("adapters must not flush")
    }
}
fn short(bytes: Vec<u8>) -> ShortIo {
    ShortIo {
        bytes,
        interrupted: false,
    }
}
const SOURCE: &str = "Version { FormatVersion 800, } Model \"Example\" {}";

#[test]
fn adapters_handle_short_io_interruptions_and_dynamic_models() {
    let model: Model<V800> = SOURCE.parse().unwrap();
    let binary = model.encode_mdx().unwrap();
    let text = model.encode_mdl().unwrap();
    let decoded: Model<V800> = mdx::from_reader(short(binary.clone())).unwrap();
    assert_eq!(decoded.encode_mdx().unwrap(), binary);
    let dynamic: DynamicModel = mdx::from_reader_with_version(short(binary.clone()), 900).unwrap();
    assert_eq!(dynamic.version(), 800);
    let fallback: DynamicModel = mdx::from_reader_with_version(b"MDLX".as_slice(), 800).unwrap();
    assert_eq!(fallback.version(), 800);
    let mut sink = short(Vec::new());
    mdx::to_writer(&mut sink, &dynamic).unwrap();
    assert_eq!(sink.bytes, binary);
    let decoded: DynamicModel = mdl::from_reader(short(text.as_bytes().to_vec())).unwrap();
    let mut sink = short(Vec::new());
    mdl::to_writer(&mut sink, &decoded).unwrap();
    assert_eq!(sink.bytes, text.as_bytes());
    let mut hive = Vec::new();
    mdl::to_writer_with_dialect(&mut hive, &decoded, mdl::Dialect::HiveWorkshop).unwrap();
    assert_eq!(
        hive,
        decoded
            .encode_mdl_with_dialect(mdl::Dialect::HiveWorkshop)
            .unwrap()
            .as_bytes()
    );
}

#[test]
fn adapters_preserve_io_and_codec_errors() {
    let error = mdx::from_reader::<Model<V800>>(Failing).unwrap_err();
    assert!(error.source().is_some());
    assert!(matches!(error, IoError::Io(error) if error.kind() == io::ErrorKind::PermissionDenied));
    assert!(
        matches!(mdl::from_reader::<Model<V800>>(Failing), Err(IoError::Io(error)) if error.kind() == io::ErrorKind::PermissionDenied)
    );
    assert!(
        matches!(mdl::from_reader::<Model<V800>>([0xff].as_slice()), Err(IoError::Io(error)) if error.kind() == io::ErrorKind::InvalidData)
    );
    assert!(matches!(
        mdx::from_reader::<u32>([0, 0, 0, 0, 1].as_slice()),
        Err(IoError::Codec(mdx::ReadError {
            kind: mdx::ReadErrorKind::TrailingBytes { .. },
            ..
        }))
    ));
    assert!(matches!(
        mdl::from_reader::<u32>(b"42 extra".as_slice()),
        Err(IoError::Codec(_))
    ));
    assert!(
        matches!(mdx::to_writer(Failing, &42u32), Err(IoError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe)
    );
    assert!(
        matches!(mdl::to_writer(Failing, &42u32), Err(IoError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe)
    );
}

#[test]
fn transport_wrappers_expose_the_original_error_as_their_source() {
    let error = mdx::from_reader::<u32>([0, 0, 0, 0, 1].as_slice()).unwrap_err();
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<mdx::ReadError>()
        .unwrap();
    assert_eq!(source.offset, 4);
    assert_eq!(
        source.kind,
        mdx::ReadErrorKind::TrailingBytes { remaining: 1 }
    );
    assert_eq!(error.to_string(), source.to_string());

    let error = mdl::to_writer(Failing, &42u32).unwrap_err();
    let source = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert_eq!(source.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn mdx_encoding_failure_leaves_sink_untouched() {
    struct Invalid;
    impl mdx::Write for Invalid {
        fn write_mdx(&self, encoder: &mut mdx::Encoder<'_>) -> Result<(), mdx::WriteError> {
            encoder.write_bytes(b"partial");
            Err(mdx::WriteError::InvalidValue {
                field: "record value",
                tag: *b"TEST",
            })
        }
    }
    let mut sink = vec![123];
    assert!(matches!(
        mdx::to_writer(&mut sink, &Invalid),
        Err(IoError::Codec(_))
    ));
    assert_eq!(sink, [123]);
}

#[test]
fn mdl_adapter_checks_block_balance() {
    struct Unbalanced;
    impl mdl::Write for Unbalanced {
        fn write_mdl<W: IoWrite>(
            &self,
            writer: &mut mdl::Writer<W>,
        ) -> Result<(), IoError<mdl::WriteError>> {
            writer.begin_block("Example")
        }
    }
    assert!(matches!(
        mdl::to_writer(Vec::new(), &Unbalanced),
        Err(IoError::Codec(mdl::WriteError::UnbalancedBlocks))
    ));
}
