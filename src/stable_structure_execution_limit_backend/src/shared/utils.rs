use candid::Principal;

pub fn principal_not_equal(x: Principal, y: Principal) -> bool {
    x != y
}

pub fn principal_equal(x: Principal, y: Principal) -> bool {
    x == y
}
