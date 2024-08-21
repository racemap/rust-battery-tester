import util from "util";
import { exec as execSync } from "child_process";

const exec = util.promisify(execSync);

async function main() {
  // build the target
  // await exec('./build.sh release')

  // convert to bin
  await exec("./elf2bin.sh");

  // TODO find a way to start WOKWI Simulator
}

main();
