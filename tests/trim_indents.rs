// Integration tests for mappingFileTrimIndents=true support.
//
// When this option is set by Datadog's Android Gradle plugin, leading whitespace
// is stripped from every member line before upload. Before the fix, such lines
// were silently dropped, leaving method_count=0 and has_line_info=false.

use proguard::{ProguardCache, ProguardMapper, ProguardMapping, StackFrame};

static MAPPING: &[u8] = include_bytes!("res/mapping-trim-indents.txt");

#[test]
fn test_trim_indents_has_line_info() {
    let mapping = ProguardMapping::new(MAPPING);
    assert!(mapping.is_valid());
    // Before the fix this returned false because all member lines were dropped.
    assert!(mapping.has_line_info());
}

#[test]
fn test_trim_indents_remap_frame() {
    let mapper = ProguardMapper::new(ProguardMapping::new(MAPPING));

    // Obfuscated frame: class "a", method "c", line 1.
    // Expands to the inline chain bar -> foo -> onClick.
    let mut frames = mapper.remap_frame(&StackFrame::new("a", "c", 1));

    assert_eq!(
        frames.next().unwrap(),
        StackFrame::with_file("com.example.Foo", "bar", 10, "Foo.java")
    );
    assert_eq!(
        frames.next().unwrap(),
        StackFrame::with_file("com.example.Foo", "foo", 20, "Foo.java")
    );
    assert_eq!(
        frames.next().unwrap(),
        StackFrame::with_file("com.example.Foo", "onClick", 30, "Foo.java")
    );
    assert_eq!(frames.next(), None);
}

#[test]
fn test_trim_indents_remap_frame_cache() {
    let mapping = ProguardMapping::new(MAPPING);
    let mut buf = Vec::new();
    ProguardCache::write(&mapping, &mut buf).unwrap();
    let cache = ProguardCache::parse(&buf).unwrap();

    let mut frames = cache.remap_frame(&StackFrame::new("a", "c", 1));

    assert_eq!(
        frames.next().unwrap(),
        StackFrame::with_file("com.example.Foo", "bar", 10, "Foo.java")
    );
    assert_eq!(
        frames.next().unwrap(),
        StackFrame::with_file("com.example.Foo", "foo", 20, "Foo.java")
    );
    assert_eq!(
        frames.next().unwrap(),
        StackFrame::with_file("com.example.Foo", "onClick", 30, "Foo.java")
    );
    assert_eq!(frames.next(), None);
}

#[test]
fn test_trim_indents_remap_stacktrace() {
    let mapper = ProguardMapper::new(ProguardMapping::new(MAPPING));

    let raw = "java.lang.RuntimeException: test\n    at a.c(Foo.java:1)\n";
    let remapped = mapper.remap_stacktrace(raw).unwrap();

    assert!(remapped.contains("com.example.Foo.bar"));
    assert!(remapped.contains("com.example.Foo.foo"));
    assert!(remapped.contains("com.example.Foo.onClick"));
}

#[test]
fn test_trim_indents_remap_stacktrace_cache() {
    let mapping = ProguardMapping::new(MAPPING);
    let mut buf = Vec::new();
    ProguardCache::write(&mapping, &mut buf).unwrap();
    let cache = ProguardCache::parse(&buf).unwrap();

    let raw = "java.lang.RuntimeException: test\n    at a.c(Foo.java:1)\n";
    let remapped = cache.remap_stacktrace(raw).unwrap();

    assert!(remapped.contains("com.example.Foo.bar"));
    assert!(remapped.contains("com.example.Foo.foo"));
    assert!(remapped.contains("com.example.Foo.onClick"));
}
