<template>
    <div
        class="group relative mx-auto w-full max-w-lg cursor-pointer rounded-lg border-2 border-dashed transition-colors"
        :class="isDragging
            ? 'border-indigo-400 bg-indigo-500/10'
            : 'border-gray-600 bg-gray-800/40 hover:border-gray-400 hover:bg-gray-800/60'"
        @click="openFilePicker"
        @dragover.prevent="isDragging = true"
        @dragenter.prevent="isDragging = true"
        @dragleave.prevent="isDragging = false"
        @drop.prevent="onDrop"
    >
        <input
            ref="fileInput"
            type="file"
            accept="audio/*"
            class="hidden"
            @change="onFileChange"
        />
        <div class="flex flex-col items-center gap-2 px-6 py-8">
            <svg class="h-8 w-8 transition-colors"
                :class="isDragging ? 'text-indigo-400' : 'text-gray-500 group-hover:text-gray-300'"
                fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round"
                    d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5m-13.5-9L12 3m0 0 4.5 4.5M12 3v13.5" />
            </svg>
            <p class="text-sm font-medium"
                :class="isDragging ? 'text-indigo-300' : 'text-gray-300'">
                <span v-if="isDragging">Drop file here</span>
                <span v-else>Drop an audio file here or <span class="text-indigo-400 underline underline-offset-2">browse</span></span>
            </p>
            <p class="text-xs text-gray-500">WAV, MP3, FLAC, OGG</p>
        </div>
    </div>
</template>

<script lang="ts">
export default {
    name: "FileUpload",
    emits: ["file-selected"],
    data() {
        return {
            isDragging: false,
        };
    },
    methods: {
        openFilePicker() {
            (this.$refs.fileInput as HTMLInputElement).click();
        },
        onFileChange(event: Event) {
            const input = event.target as HTMLInputElement;
            if (input.files && input.files.length > 0) {
                this.$emit("file-selected", input.files[0]);
            }
        },
        onDrop(event: DragEvent) {
            this.isDragging = false;
            const files = event.dataTransfer?.files;
            if (files && files.length > 0) {
                this.$emit("file-selected", files[0]);
            }
        },
    },
};
</script>