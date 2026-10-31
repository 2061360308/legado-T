export interface PickedWebFile {
  file: File;
  relativePath: string;
}

function acceptFromFilters(
  filters?: { name: string; extensions: string[] }[],
): string {
  if (!filters?.length) return "";
  return filters
    .flatMap((filter) => filter.extensions)
    .filter(Boolean)
    .map((ext) => `.${ext.replace(/^\./, "")}`)
    .join(",");
}

export function pickWebFiles(options: {
  directory?: boolean;
  multiple?: boolean;
  filters?: { name: string; extensions: string[] }[];
} = {}): Promise<PickedWebFile[]> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = options.multiple ?? true;
    input.accept = acceptFromFilters(options.filters);
    if (options.directory) {
      input.setAttribute("webkitdirectory", "");
      input.setAttribute("directory", "");
    }
    input.style.position = "fixed";
    input.style.left = "-9999px";
    input.style.top = "-9999px";
    const cleanup = () => {
      input.remove();
      resolve([]);
    };
    input.addEventListener("change", () => {
      const files = Array.from(input.files ?? []);
      input.remove();
      resolve(
        files.map((file) => ({
          file,
          relativePath: (file as File & { webkitRelativePath?: string }).webkitRelativePath || file.name,
        })),
      );
    });
    input.addEventListener("cancel", cleanup);
    document.body.appendChild(input);
    input.click();
  });
}

export function pickWebFile(options: {
  filters?: { name: string; extensions: string[] }[];
} = {}): Promise<PickedWebFile | null> {
  return pickWebFiles({ ...options, multiple: false }).then((files) => files[0] ?? null);
}
