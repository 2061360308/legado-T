import { platform } from "@/runtime";

export async function saveTextFile(
  defaultPath: string,
  content: string,
  filterName = "JSON",
): Promise<boolean> {
  const path = await platform.dialog.save({
    defaultPath,
    filters: [{ name: filterName, extensions: ["json"] }],
  });
  if (!path) return false;
  await platform.files.writeFile(path, new TextEncoder().encode(content));
  return true;
}
