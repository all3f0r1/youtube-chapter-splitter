/// Benchmarks de performance pour YouTube Chapter Splitter
///
/// Pour exécuter : cargo bench
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use youtube_chapter_splitter::chapters::Chapter;
use youtube_chapter_splitter::utils::{
    clean_folder_name, format_duration, parse_artist_album, sanitize_title,
};

fn bench_clean_folder_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("clean_folder_name");

    let test_cases = vec![
        ("Simple Title", "Simple Title"),
        (
            "MARIGOLD - Oblivion Gate [Full Album] (70s Rock)",
            "MARIGOLD - Oblivion Gate [Full Album] (70s Rock)",
        ),
        (
            "Artist_Name - Album_Title [2024] (Remastered)",
            "Artist_Name - Album_Title [2024] (Remastered)",
        ),
        (
            "Very Long Title With Many Words And Brackets [Full Album] [Remastered] [2024]",
            "Very Long Title With Many Words And Brackets [Full Album] [Remastered] [2024]",
        ),
    ];

    for (input, _) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(input), &input, |b, &input| {
            b.iter(|| clean_folder_name(black_box(input)));
        });
    }

    group.finish();
}

fn bench_parse_artist_album(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_artist_album");

    let test_cases = vec![
        "Pink Floyd - Dark Side of the Moon [1973]",
        "MARIGOLD - Oblivion Gate [Full Album]",
        "Artist | Album Name",
        "Just A Title Without Separator",
    ];

    for input in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(input), &input, |b, &input| {
            b.iter(|| parse_artist_album(black_box(input), black_box("TestChannel")));
        });
    }

    group.finish();
}

fn bench_sanitize_title(c: &mut Criterion) {
    let mut group = c.benchmark_group("sanitize_title");

    let test_cases = vec![
        "1 - Song Name",
        "Track 5: Another Song",
        "01. Song With Number",
        "Invalid/Characters:Here",
        "Very Long Song Title With Many Characters And Special Chars/:*?\"<>|",
    ];

    for input in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(input), &input, |b, &input| {
            b.iter(|| sanitize_title(black_box(input)));
        });
    }

    group.finish();
}

fn bench_format_duration(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_duration");

    let test_cases = vec![
        ("short", 45.0),
        ("medium", 343.0),
        ("long", 3661.0),
        ("very_long", 36000.0),
    ];

    for (name, duration) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &duration,
            |b, &duration| {
                b.iter(|| format_duration(black_box(duration)));
            },
        );
    }

    group.finish();
}

fn bench_chapter_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("chapter_operations");

    // Benchmark création de chapitre
    group.bench_function("create_chapter", |b| {
        b.iter(|| {
            Chapter::new(
                black_box("Test Chapter".to_string()),
                black_box(0.0),
                black_box(180.0),
            )
        });
    });

    // Benchmark calcul de durée
    let chapter = Chapter::new("Test".to_string(), 0.0, 180.0);
    group.bench_function("chapter_duration", |b| {
        b.iter(|| chapter.duration());
    });

    // Benchmark sanitization de titre
    let chapter = Chapter::new("1 - Test Song".to_string(), 0.0, 180.0);
    group.bench_function("chapter_sanitize_title", |b| {
        b.iter(|| chapter.sanitize_title());
    });

    group.finish();
}

fn bench_multiple_chapters(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_chapters");

    // Benchmark création de multiples chapitres
    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut chapters = Vec::new();
                for i in 0..count {
                    chapters.push(Chapter::new(
                        format!("Chapter {}", i),
                        i as f64 * 180.0,
                        (i + 1) as f64 * 180.0,
                    ));
                }
                chapters
            });
        });
    }

    group.finish();
}

fn bench_regex_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("regex_operations");

    // Benchmark avec regex pré-compilée (comme dans le code actuel)
    use once_cell::sync::Lazy;
    use regex::Regex;

    static RE_TEST: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[.*?\]|\(.*?\)").unwrap());

    let test_string = "MARIGOLD - Oblivion Gate [Full Album] (70s Rock)";

    group.bench_function("precompiled_regex", |b| {
        b.iter(|| RE_TEST.replace_all(black_box(test_string), ""));
    });

    // Benchmark avec regex compilée à chaque fois (ancien code)
    group.bench_function("runtime_regex", |b| {
        b.iter(|| {
            let re = Regex::new(r"\[.*?\]|\(.*?\)").unwrap();
            re.replace_all(black_box(test_string), "")
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_clean_folder_name,
    bench_parse_artist_album,
    bench_sanitize_title,
    bench_format_duration,
    bench_chapter_operations,
    bench_multiple_chapters,
    bench_regex_compilation
);

criterion_main!(benches);
