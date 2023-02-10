use std::{ops::AddAssign, time::Duration};
use futures::{join, future::select};
use spiderweb::{sync::Mutex, task::sleep};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn basic () {
    let value = Mutex::new(2);

    let lhs = async {
        let mut value = value.lock().await;
        value.add_assign(1);
    };
    let rhs = async {
        let mut value = value.lock().await;
        value.add_assign(1);
    };

    join!(lhs, rhs);
    spiderweb::println!("{} = 4", value.into_inner());
}

#[wasm_bindgen_test]
async fn drop_before_complete () {
    let mut value = Mutex::new(2);

    let lhs = async {
        let mut value = value.lock().await;
        spiderweb::println!("Lock 1 acquired");
        sleep(Duration::from_secs(1)).await;
        value.add_assign(1);
        drop(value);
        spiderweb::println!("Lock 1 dropped");
    };
    let rhs = async {
        let mut value = value.lock().await;
        spiderweb::println!("Lock 2 acquired");
        sleep(Duration::from_secs(1)).await;
        value.add_assign(1);
        drop(value);
        spiderweb::println!("Lock 2 acquired");
    };

    select(Box::pin(lhs), Box::pin(rhs)).await;
    spiderweb::println!("{} = 3", value.get_mut());
}