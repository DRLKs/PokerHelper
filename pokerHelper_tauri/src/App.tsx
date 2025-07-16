import "./styles/global.css";
import "./styles/App.css";
import { PokerHand, PlayerHand, LanguageSelector, AnalysisDisplay } from "./components";
import { LanguageProvider, useLanguage } from "./contexts/LanguageContext";
import { PokerGameProvider } from "./contexts/PokerGameContext";

function AppContent() {
  const { t } = useLanguage();

  return (
    <main className="container">
      <div className="header">
        <h1 className="title">{t.appTitle}</h1>
        <LanguageSelector />
      </div>
      
      <PlayerHand />
      <PokerHand />
      <AnalysisDisplay />
    </main>
  );
}

function App() {
  return (
    <LanguageProvider>
      <PokerGameProvider>
        <AppContent />
      </PokerGameProvider>
    </LanguageProvider>
  );
}

export default App;
