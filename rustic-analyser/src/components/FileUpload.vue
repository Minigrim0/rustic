<template>
    <div class="file-upload">
        <h3>Upload Audio File</h3>
        <div class="upload-container">
            <input
                type="file"
                id="audio-file"
                accept="audio/*"
                @change="onFileChange"
            />
            <label for="audio-file">
                <span class="upload-icon">📂</span>
                <span class="upload-text">Choose a file or drag it here</span>
            </label>
            <p class="file-info">Supported formats: WAV, MP3, FLAC, OGG</p>
        </div>
    </div>
</template>

<script>
/**
 * FileUpload component
 *
 * Allows users to select audio files for analysis
 * Emits the selected file to the parent component
 */
export default {
    name: "FileUpload",
    emits: ["file-selected"],
    methods: {
        /**
         * Handles file selection event
         * @param {Event} event - The change event from the file input
         */
        onFileChange(event) {
            const files = event.target.files;
            if (files && files.length > 0) {
                const file = files[0];
                this.$emit("file-selected", file);
            }
        },
    },
};
</script>
