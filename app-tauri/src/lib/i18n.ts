import en from '../locales/en.json';

type LocaleDict = Record<string, any>;

const locales: Record<string, LocaleDict> = {
    en
};

let currentLocale = $state('en');

export function setLocale(locale: string) {
    if (locales[locale]) {
        currentLocale = locale;
    }
}

export function t(key: string): string {
    const keys = key.split('.');
    let value: any = locales[currentLocale];
    for (const k of keys) {
        if (value && typeof value === 'object') {
            value = value[k];
        } else {
            return key;
        }
    }
    return typeof value === 'string' ? value : key;
}
