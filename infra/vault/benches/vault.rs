use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use getrandom::fill;
use mhub_vault::prelude::*;

#[vault_model]
struct BenchValue {
    value: String,
}

fn bench_seal_unseal(c: &mut Criterion) {
    let mut group = c.benchmark_group("seal_unseal");

    let vault = Vault::<Aes>::builder()
        .derived_keys("bench-ikm", "bench-salt", "bench-id")
        .unwrap()
        .compression(true)
        .build()
        .unwrap();

    let sizes = [("256B", 256usize), ("4KB", 4 * 1024), ("64KB", 64 * 1024)];

    for (label, size) in sizes {
        let mut data = vec![0u8; size];
        fill(&mut data).expect("System RNG unavailable for benchmark data");

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("seal_local_json", label), &data, |b, d| {
            b.iter(|| {
                let value = String::from_utf8_lossy(d);
                let wrapped = BenchValue { value: value.to_string() };
                vault.seal::<Local, _>(&wrapped).unwrap();
            });
        });

        let sealed = vault.seal_bytes::<Local>(data.clone(), b"ctx").expect("seal_bytes failed");

        group.bench_with_input(BenchmarkId::new("unseal_local_raw", label), &sealed, |b, s| {
            b.iter(|| {
                let _ = vault.unseal_bytes::<Local>(s, b"ctx").unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_seal_unseal);
criterion_main!(benches);
