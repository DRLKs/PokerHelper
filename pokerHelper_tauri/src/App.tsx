import "./styles/global.css";
import "./styles/App.css";
import { PokerHand, PlayerHand, LanguageSelector, AnalysisDisplay, AnalysisOptions } from "./components";
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
      
      <div className="main-content">
        <div className="analysis-section">
          <AnalysisDisplay />
        </div>
        
        <div className="game-section">
          <PokerHand />
          <PlayerHand />
          <AnalysisOptions />
        </div>
      </div>
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
