import "./style.css";
import {initStorage} from "./storage.ts";
import {initJuno} from "@junobuild/core";

document.addEventListener(
    "DOMContentLoaded",
    async () => {
        await initJuno({
            satelliteId: "ajuq4-ruaaa-aaaaa-qaaga-cai",
            localIdentityCanisterId: "rrkah-fqaaa-aaaaa-aaaaq-cai"
        });
    },
    { once: true }
);

initStorage();