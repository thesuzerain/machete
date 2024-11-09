declare global {
    interface ImportMetaEnv {
        PUBLIC_API_URL: string
    }
}

export const API_URL = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:8000'; 