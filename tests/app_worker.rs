use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use hyprbar::app::WorkerHandle;

#[test]
fn drop_requests_shutdown_and_joins_the_worker() {
    let finished = Arc::new(AtomicBool::new(false));
    let finished_worker = finished.clone();

    let handle = WorkerHandle::spawn("test-worker-join", move |shutdown| {
        while !shutdown.should_stop() {
            shutdown.sleep(Duration::from_millis(20));
        }

        finished_worker.store(true, Ordering::Release);
    })
    .expect("el worker de prueba debería arrancar");

    drop(handle);

    // Drop joinea: al volver, el cuerpo del worker ya terminó entero.
    assert!(finished.load(Ordering::Acquire));
}

#[test]
fn sleep_is_interrupted_by_shutdown() {
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_worker = interrupted.clone();

    let started_at = Instant::now();

    let handle = WorkerHandle::spawn("test-worker-sleep", move |shutdown| {
        // Sin interrupción esto colgaría el join por 30 segundos.
        let was_interrupted = shutdown.sleep(Duration::from_secs(30));

        interrupted_worker.store(was_interrupted, Ordering::Release);
    })
    .expect("el worker de prueba debería arrancar");

    drop(handle);

    assert!(interrupted.load(Ordering::Acquire), "sleep debería reportar la interrupción");

    // Margen generoso para CI lenta, pero muy por debajo de los 30s.
    assert!(started_at.elapsed() < Duration::from_secs(5), "el shutdown no interrumpió el sleep a tiempo");
}

#[test]
fn a_panicking_worker_does_not_poison_the_drop() {
    let handle = WorkerHandle::spawn("test-worker-panic", |_shutdown| {
        panic!("panic de prueba");
    })
    .expect("el worker de prueba debería arrancar");

    // Drop debe absorber el panic del join (solo loguea).
    drop(handle);
}
