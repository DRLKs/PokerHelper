import { useLanguage } from '../contexts/LanguageContext';
import { Language } from '../types/Language';

const languageOptions = [
  { code: 'es' as Language, name: 'Español', flag: '🇪🇸' },
  { code: 'en' as Language, name: 'English', flag: '🇺🇸' },
  { code: 'fr' as Language, name: 'Français', flag: '🇫🇷' }
];

export const LanguageSelector = () => {
  const { currentLanguage, setLanguage, t } = useLanguage();

  return (
    <div className="language-selector">
      <select 
        value={currentLanguage} 
        onChange={(e) => setLanguage(e.target.value as Language)}
        className="language-select"
      >
        {languageOptions.map((option) => (
          <option key={option.code} value={option.code}>
            {option.flag} {option.name}
          </option>
        ))}
      </select>
    </div>
  );
};
