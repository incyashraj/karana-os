import React from 'react';
import { Sparkles, ArrowRight } from 'lucide-react';

export interface Suggestion {
  text: string;
  action?: string;
  icon?: string;
}

export interface SuggestionChipsProps {
  suggestions: Suggestion[];
  onSuggestionClick: (suggestion: Suggestion) => void;
  title?: string;
  className?: string;
}

export const SuggestionChips: React.FC<SuggestionChipsProps> = ({
  suggestions,
  onSuggestionClick,
  title = 'Suggestions',
  className = '',
}) => {
  if (suggestions.length === 0) return null;

  return (
    <div className={`space-y-2 ${className}`}>
      <div className="flex items-center gap-2 text-sm font-medium text-gray-600">
        <Sparkles className="w-4 h-4 text-purple-500" />
        <span>{title}</span>
      </div>
      <div className="flex flex-wrap gap-2">
        {suggestions.map((suggestion, index) => (
          <button
            key={index}
            onClick={() => onSuggestionClick(suggestion)}
            className="group inline-flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-indigo-50 to-purple-50 hover:from-indigo-100 hover:to-purple-100 border border-indigo-200 rounded-full text-sm font-medium text-gray-700 transition-all hover:shadow-md hover:scale-105 active:scale-95"
          >
            {suggestion.icon && <span>{suggestion.icon}</span>}
            <span>{suggestion.text}</span>
            <ArrowRight className="w-3 h-3 text-gray-400 group-hover:text-indigo-600 transition-colors" />
          </button>
        ))}
      </div>
    </div>
  );
};
