import { writeFileSync } from "node:fs";
import { generateManifest } from "material-icon-theme";
import { join } from "node:path";
import { getTheme } from "./theme";
import fs from "node:fs";

const manifest = generateManifest();
const darkTheme = getTheme(manifest, {
  name: "DX Icon",
  appearance: "dark",
});
const lightTheme = getTheme(manifest, {
  name: "DX Icon Light",
  appearance: "light",
});

const zedManifest = {
  $schema: "https://zed.dev/schema/icon_themes/v0.2.0.json",
  name: "DX Icon",
  author: "DX, based on Zed Material Icon Theme",
  themes: [darkTheme, lightTheme],
};

writeFileSync(
  join(__dirname, "../icon_themes", "dx-icon.json"),
  JSON.stringify(zedManifest, null, 2),
);

// Copy icons from node_modules to the icons directory
const iconsSourceDir = join(
  __dirname,
  "../node_modules/material-icon-theme/icons",
);
const iconsDestDir = join(__dirname, "../icons");
if (!fs.existsSync(iconsDestDir)) {
  fs.mkdirSync(iconsDestDir, { recursive: true });
}
fs.readdirSync(iconsSourceDir).forEach((file) => {
  const sourceFile = join(iconsSourceDir, file);
  const destFile = join(iconsDestDir, file);
  fs.copyFileSync(sourceFile, destFile);
});
console.log("DX Icon theme generated successfully.");
