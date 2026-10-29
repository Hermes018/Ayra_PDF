export interface FaqItem {
    id: string;
    feature: string;
    question: string;
    answer: string;
    usage: string;
    outputs: string;
    offlineBoundaries: string;
}

export interface FaqCatalog {
    locale: string;
    language: string;
    title: string;
    searchPlaceholder: string;
    emptySearch: string;
    items: FaqItem[];
}

export type SupportedFaqLang = 'en' | 'bn' | 'fr' | 'zh' | 'ja' | 'ta';

export const SUPPORTED_LANGUAGES: { code: SupportedFaqLang; label: string }[] = [
    { code: 'en', label: 'English' },
    { code: 'bn', label: 'বাংলা (Bengali)' },
    { code: 'fr', label: 'Français (French)' },
    { code: 'zh', label: '简体中文 (Chinese)' },
    { code: 'ja', label: '日本語 (Japanese)' },
    { code: 'ta', label: 'தமிழ் (Tamil)' },
];

class FaqStore {
    currentLang = $state<SupportedFaqLang>('en');
    searchQuery = $state<string>('');
    isLoading = $state<boolean>(false);
    catalog = $state<FaqCatalog | null>(null);
    isOpen = $state<boolean>(false);

    filteredItems = $derived.by(() => {
        if (!this.catalog) return [];
        const q = this.searchQuery.trim().toLowerCase();
        if (!q) return this.catalog.items;
        return this.catalog.items.filter(item => 
            item.feature.toLowerCase().includes(q) ||
            item.question.toLowerCase().includes(q) ||
            item.answer.toLowerCase().includes(q) ||
            item.usage.toLowerCase().includes(q) ||
            item.outputs.toLowerCase().includes(q) ||
            item.offlineBoundaries.toLowerCase().includes(q)
        );
    });

    constructor() {
        this.loadLanguage('en');
    }

    async loadLanguage(lang: SupportedFaqLang) {
        this.isLoading = true;
        try {
            const data = await import(`./faq/${lang}.json`);
            this.catalog = (data.default || data) as FaqCatalog;
            this.currentLang = lang;
        } catch (err) {
            console.warn(`[FAQ] Failed to lazy-load language '${lang}', falling back to 'en':`, err);
            try {
                const fallback = await import('./faq/en.json');
                this.catalog = (fallback.default || fallback) as FaqCatalog;
                this.currentLang = 'en';
            } catch (fallbackErr) {
                console.error('[FAQ] Critical fallback failure:', fallbackErr);
            }
        } finally {
            this.isLoading = false;
        }
    }

    open() {
        this.isOpen = true;
    }

    close() {
        this.isOpen = false;
        this.searchQuery = '';
    }

    toggle() {
        if (this.isOpen) {
            this.close();
        } else {
            this.open();
        }
    }
}

export const faqStore = new FaqStore();
