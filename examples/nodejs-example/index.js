// index.js
import * as example from "./pkg/nodejs_example.js";

(async () => {
  console.log("Sleeping JS");
  await example.sleep_test();
  console.log("Slept JS");

  console.log("Interval JS");
  await example.interval_test();
  console.log("Interval End JS");

  console.log("Timeout Start JS");
  await example.timeout_test();
  console.log("Timeout Finished JS");

  console.log("Tokio macros start JS");
  await example.tokio_macros_test();
  console.log("Tokio macros end JS");
  process.exit(0);
})().catch((e) => {
  console.error(e);
});
