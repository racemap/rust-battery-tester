import fs from "fs";
import { execSync } from "child_process";

function versionCompare(
  v1: string,
  v2: string,
  options?: { lexicographical?: boolean; zeroExtend?: boolean }
) {
  const lexicographical = options && options.lexicographical;
  const zeroExtend = options && options.zeroExtend;
  let v1parts = v1.split(".");
  let v2parts = v2.split(".");
  console.log("versionCompare", v1parts, v2parts);

  function isValidPart(x) {
    return (lexicographical ? /^\d+[A-Za-z]*$/ : /^\d+$/).test(x);
  }

  if (!v1parts.every(isValidPart) || !v2parts.every(isValidPart)) {
    return NaN;
  }

  if (zeroExtend) {
    while (v1parts.length < v2parts.length) v1parts.push("0");
    while (v2parts.length < v1parts.length) v2parts.push("0");
  }

  if (!lexicographical) {
    v1parts = v1parts.map(String);
    v2parts = v2parts.map(String);
  }

  for (let i = 0; i < v1parts.length; ++i) {
    if (v2parts.length == i) {
      return 1;
    }

    if (v1parts[i] == v2parts[i]) {
      continue;
    } else if (Number.parseInt(v1parts[i]) > Number.parseInt(v2parts[i])) {
      return 1;
    } else {
      return -1;
    }
  }

  if (v1parts.length != v2parts.length) {
    return -1;
  }

  return 0;
}

const gitConfig = ".git/config";
const gitVersion = "" + execSync("git --version");
const pGitVersion = gitVersion.replace("git version ", "").replace("\n", "");

console.log(`Try to mod ${gitConfig}.`);
console.log(
  `The hooksPath feature is working as of git 2.9.0. You have ${gitVersion}`
);
if (versionCompare("2.9.0", pGitVersion) > 0) {
  console.warn("Your git is too old. Please Update!");
} else {
  console.info("Your git is right. Continue!");
}

if (fs.existsSync(gitConfig)) {
  const text = fs.readFileSync(gitConfig).toString();
  const hasHooksPath = text.indexOf("hooksPath") > -1;

  if (hasHooksPath) {
    console.info("|-> hooksPath already set!");
  } else {
    console.info("|-> hooksPath missing. Try to set!");
    const lines = text.split("\n");
    const newLines = [];
    if (lines != null) {
      lines.forEach((line, index) => {
        newLines.push(line);
        if (line.indexOf("[core]") > -1) {
          newLines.push("\thooksPath = .hooks");
          console.info("|-> hooksPath set in line =", index + 1);
        }
      });
    }
    fs.writeFileSync(gitConfig, newLines.join("\n"));
  }
} else {
  console.warn("|-> File does not exist");
}
