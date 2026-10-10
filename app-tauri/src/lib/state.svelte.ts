class AppState {
    activeFiles = $state<string[]>([]);
    
    get currentFile(): string | null {
        return this.activeFiles.length > 0 ? this.activeFiles[0] : null;
    }

    addFiles(paths: string[]) {
        for (const path of paths) {
            if (!this.activeFiles.includes(path)) {
                this.activeFiles.push(path);
            }
        }
    }

    clear() {
        this.activeFiles = [];
    }
}

export const appState = new AppState();
