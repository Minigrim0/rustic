<template>
    <aside class="flex h-full w-12 flex-col items-center border-r border-gray-200 bg-gray-100 py-2 select-none dark:border-white/10 dark:bg-gray-900">
        <router-link v-for="item in navItems" :key="item.name" :to="item.to" :title="item.label"
            class="flex h-10 w-10 items-center justify-center rounded-md transition-colors"
            :class="isActive(item.to)
                ? 'bg-gray-300/60 text-gray-900 dark:bg-white/15 dark:text-white'
                : 'text-gray-500 hover:bg-gray-200/60 hover:text-gray-700 dark:text-gray-500 dark:hover:bg-white/5 dark:hover:text-gray-300'
            ">
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round" :d="item.icon" />
            </svg>
        </router-link>

        <div class="mt-auto flex flex-col items-center">
            <router-link to="/settings" title="Settings"
                class="flex h-10 w-10 items-center justify-center rounded-md transition-colors"
                :class="isActive('/settings')
                    ? 'bg-gray-300/60 text-gray-900 dark:bg-white/15 dark:text-white'
                    : 'text-gray-500 hover:bg-gray-200/60 hover:text-gray-700 dark:text-gray-500 dark:hover:bg-white/5 dark:hover:text-gray-300'
                ">
                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                    <path stroke-linecap="round" stroke-linejoin="round" :d="settingsIcon" />
                </svg>
            </router-link>
            <button @click="toggleTheme" :title="themeLabel"
                class="flex h-10 w-10 items-center justify-center rounded-md text-gray-500 transition-colors hover:bg-gray-200/60 hover:text-gray-700 dark:text-gray-500 dark:hover:bg-white/5 dark:hover:text-gray-300">
                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                    <path stroke-linecap="round" stroke-linejoin="round" :d="themeIconPath" />
                </svg>
            </button>
        </div>
    </aside>
</template>

<script lang="ts">
export default {
    name: "Navbar",
    data() {
        return {
            currentTheme: "dark",
            navItems: [
                {
                    name: "analyzer",
                    to: "/",
                    label: "Analyzer",
                    // waveform icon
                    icon: "M3 12h2l3-8 4 16 4-12 3 6h2",
                },
            ],
            settingsIcon: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.248a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.212-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28ZM15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z",
        };
    },
    computed: {
        themeIconPath(): string {
            return this.currentTheme === "light"
                // moon
                ? "M21.752 15.002A9.72 9.72 0 0 1 18 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 0 0 3 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 0 0 9.002-5.998Z"
                // sun
                : "M12 3v2.25m6.364.386-1.591 1.591M21 12h-2.25m-.386 6.364-1.591-1.591M12 18.75V21m-4.773-4.227-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0Z";
        },
        themeLabel(): string {
            return this.currentTheme === "light" ? "Dark Mode" : "Light Mode";
        },
    },
    mounted() {
        this.initializeTheme();
    },
    methods: {
        isActive(to: string): boolean {
            return this.$route.path === to;
        },

        initializeTheme() {
            const savedTheme = localStorage.getItem("rustic-theme");
            const systemPrefersDark = window.matchMedia(
                "(prefers-color-scheme: dark)",
            ).matches;

            const initialTheme = savedTheme || (systemPrefersDark ? "dark" : "light");
            this.setTheme(initialTheme);

            window
                .matchMedia("(prefers-color-scheme: dark)")
                .addEventListener("change", (e) => {
                    if (!localStorage.getItem("rustic-theme")) {
                        this.setTheme(e.matches ? "dark" : "light");
                    }
                });
        },

        setTheme(theme: string) {
            this.currentTheme = theme;
            document.documentElement.classList.toggle("dark", theme === "dark");
            localStorage.setItem("rustic-theme", theme);
        },

        toggleTheme() {
            this.setTheme(this.currentTheme === "light" ? "dark" : "light");
        },
    },
};
</script>