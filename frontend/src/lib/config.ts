declare global {
    interface ImportMetaEnv {
        VITE_PUBLIC_API_URL: string
    }
}

export const API_URL = import.meta.env.VITE_PUBLIC_API_URL ?? 'http://localhost:8000'; 