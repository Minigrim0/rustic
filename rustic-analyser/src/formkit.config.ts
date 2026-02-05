import type { DefaultConfigOptions } from "@formkit/vue";

const textClasses =
    "block w-full rounded border border-gray-200 bg-gray-50 px-2 py-1 text-sm text-gray-700 focus:border-indigo-400 focus:outline-none dark:border-white/10 dark:bg-gray-900 dark:text-gray-300";

const classMap: Record<string, Record<string, string>> = {
    global: {
        outer: "mb-3",
        wrapper: "",
        label: "mb-0.5 block text-[11px] font-medium text-gray-600 dark:text-gray-400",
        inner: "",
        input: textClasses,
        help: "mt-0.5 text-[10px] text-gray-400 dark:text-gray-500",
        messages: "mt-1 list-none p-0",
        message: "text-[10px] text-red-500 dark:text-red-400",
    },
    checkbox: {
        wrapper: "flex items-center gap-2",
        inner: "flex items-center",
        input: "h-3.5 w-3.5 rounded border-gray-300 text-indigo-500 focus:ring-indigo-400 dark:border-white/20 dark:bg-gray-900",
        label: "mb-0 text-[11px] font-medium text-gray-600 dark:text-gray-400",
    },
    select: {
        input: textClasses + " appearance-none",
    },
};

const config: DefaultConfigOptions = {
    config: {
        rootClasses(sectionKey: string, node: any) {
            const type = node?.props?.type ?? "global";
            const typeClasses = classMap[type] ?? {};
            const globalClasses = classMap.global ?? {};
            const classes = typeClasses[sectionKey] ?? globalClasses[sectionKey] ?? "";
            return { [classes]: true };
        },
    },
};

export default config;