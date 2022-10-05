import { createRequire } from "https://deno.land/std@0.158.0/node/module.ts";
const require = createRequire(import.meta.url);
const module = require("./sub.js");
console.log(await module.sub(3, 1));
