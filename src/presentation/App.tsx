import React, { useState } from 'react';
import { usePoker } from '../application/usePoker';
import { CardView } from './components/CardView';
import { CardSelector } from './components/CardSelector';
import { Card } from '../domain/types';

const App = () => {
  const { 
    myCards, addMyCard, removeMyCard, 
    communityCards, addCommunityCard, removeCommunityCard,
    numOpponents, setNumOpponents,
    equity, calculate, loading, error 
  } = usePoker();

  const [target, setTarget] = useState<'hand' | 'community' | null>('hand');

  const handleCardSelect = (card: Card) => {
    if (target === 'hand') {
      if (myCards.length < 2) addMyCard(card);
    } else if (target === 'community') {
      if (communityCards.length < 5) addCommunityCard(card);
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8 flex flex-col items-center font-sans">
      <h1 className="text-3xl font-bold mb-8 text-poker-accent tracking-widest">POKER HELPER</h1>

      <div className="w-full max-w-5xl grid grid-cols-1 lg:grid-cols-2 gap-8">
        
        {/* Left Column: Game State */}
        <div className="space-y-8">
          
          {/* My Hand */}
          <div 
            className={`p-6 rounded-xl border-2 transition-all cursor-pointer ${target === 'hand' ? 'border-poker-accent bg-gray-800 shadow-lg shadow-poker-accent/10' : 'border-gray-700 bg-gray-800/30 hover:bg-gray-800/50'}`}
            onClick={() => setTarget('hand')}
          >
            <h2 className="text-xl font-semibold mb-4 text-gray-300 flex justify-between items-center">
              MY HAND 
              <span className={`text-xs px-2 py-1 rounded ${target === 'hand' ? 'bg-poker-accent text-black' : 'bg-gray-700 text-gray-400'}`}>
                {target === 'hand' ? 'ACTIVE' : 'SELECT'}
              </span>
            </h2>
            <div className="flex gap-4 justify-center">
              {myCards.map((card, i) => (
                <CardView key={i} card={card} onClick={() => removeMyCard(i)} />
              ))}
              {Array.from({ length: 2 - myCards.length }).map((_, i) => (
                <div key={i} className="w-16 h-24 border-2 border-dashed border-gray-600 rounded-lg flex items-center justify-center text-gray-600">
                  +
                </div>
              ))}
            </div>
          </div>

          {/* Community Cards */}
          <div 
            className={`p-6 rounded-xl border-2 transition-all cursor-pointer ${target === 'community' ? 'border-poker-accent bg-gray-800 shadow-lg shadow-poker-accent/10' : 'border-gray-700 bg-gray-800/30 hover:bg-gray-800/50'}`}
            onClick={() => setTarget('community')}
          >
            <h2 className="text-xl font-semibold mb-4 text-gray-300 flex justify-between items-center">
              COMMUNITY 
              <span className={`text-xs px-2 py-1 rounded ${target === 'community' ? 'bg-poker-accent text-black' : 'bg-gray-700 text-gray-400'}`}>
                {target === 'community' ? 'ACTIVE' : 'SELECT'}
              </span>
            </h2>
            <div className="flex gap-4 flex-wrap justify-center">
              {communityCards.map((card, i) => (
                <CardView key={i} card={card} onClick={() => removeCommunityCard(i)} />
              ))}
              {Array.from({ length: 5 - communityCards.length }).map((_, i) => (
                <div key={i} className="w-16 h-24 border-2 border-dashed border-gray-600 rounded-lg flex items-center justify-center text-gray-600">
                  +
                </div>
              ))}
            </div>
          </div>

          {/* Opponents */}
          <div className="p-6 rounded-xl border border-gray-700 bg-gray-800/50">
            <h2 className="text-xl font-semibold mb-4 text-gray-300">OPPONENTS</h2>
            <div className="flex items-center gap-4">
              <input 
                type="range" 
                min="1" max="9" 
                value={numOpponents} 
                onChange={(e) => setNumOpponents(parseInt(e.target.value))}
                className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-poker-accent"
              />
              <span className="text-2xl font-bold w-8 text-center text-poker-accent">{numOpponents}</span>
            </div>
          </div>

        </div>

        {/* Right Column: Controls & Stats */}
        <div className="space-y-8 flex flex-col">
          
          {/* Card Selector */}
          <div className="flex-grow">
             <CardSelector onSelect={handleCardSelect} />
          </div>

          {/* Action */}
          <button
            onClick={calculate}
            disabled={loading || myCards.length !== 2}
            className={`
              w-full py-4 rounded-xl font-bold text-xl tracking-wider transition-all
              ${loading || myCards.length !== 2 
                ? 'bg-gray-700 text-gray-500 cursor-not-allowed' 
                : 'bg-poker-green hover:bg-green-600 text-white shadow-lg hover:shadow-green-900/50 transform hover:-translate-y-1'}
            `}
          >
            {loading ? 'CALCULATING...' : 'CALCULATE EQUITY'}
          </button>

          {/* Result */}
          {equity !== null && (
            <div className="p-6 rounded-xl bg-gray-800 border border-gray-700 text-center animate-fade-in shadow-2xl">
              <h3 className="text-gray-400 mb-2 uppercase text-sm tracking-widest">Win Probability</h3>
              <div className="text-6xl font-bold text-poker-accent drop-shadow-lg">
                {(equity * 100).toFixed(2)}%
              </div>
            </div>
          )}

          {error && (
            <div className="p-4 rounded-lg bg-red-900/20 border border-red-700/50 text-red-200 text-center">
              {error}
            </div>
          )}

        </div>
      </div>
    </div>
  );
};

export default App;
