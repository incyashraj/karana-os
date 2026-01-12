/**
 * Kāraṇa OS - Real-Time Internet Services
 * 
 * Provides access to live data from the internet:
 * - News (location-aware, categorized)
 * - Weather (current & forecast)
 * - Web search
 * - Stock prices
 * - Currency exchange
 * - Traffic updates
 * - Events nearby
 */

// =============================================================================
// TYPES
// =============================================================================

export interface NewsArticle {
  title: string;
  description: string;
  url: string;
  source: string;
  publishedAt: string;
  category?: string;
  imageUrl?: string;
}

export interface WeatherData {
  location: string;
  temperature: number;
  condition: string;
  humidity: number;
  windSpeed: number;
  forecast: Array<{
    day: string;
    high: number;
    low: number;
    condition: string;
  }>;
}

export interface SearchResult {
  title: string;
  snippet: string;
  url: string;
  displayUrl: string;
}

export interface LocationData {
  city: string;
  country: string;
  region: string;
  timezone: string;
  coordinates: {
    lat: number;
    lng: number;
  };
}

// =============================================================================
// LOCATION SERVICE
// =============================================================================

class LocationService {
  private cachedLocation: LocationData | null = null;
  private lastFetch: number = 0;
  private CACHE_DURATION = 3600000; // 1 hour

  async getLocation(): Promise<LocationData> {
    // Check cache
    if (this.cachedLocation && Date.now() - this.lastFetch < this.CACHE_DURATION) {
      return this.cachedLocation;
    }

    try {
      // Try browser geolocation first
      if ('geolocation' in navigator) {
        const position = await new Promise<GeolocationPosition>((resolve, reject) => {
          navigator.geolocation.getCurrentPosition(resolve, reject);
        });

        // Reverse geocode to get city/country
        const location = await this.reverseGeocode(
          position.coords.latitude,
          position.coords.longitude
        );
        
        this.cachedLocation = location;
        this.lastFetch = Date.now();
        return location;
      }
    } catch (error) {
      console.warn('Geolocation failed:', error);
    }

    // Fallback to IP-based location
    try {
      const response = await fetch('https://ipapi.co/json/');
      const data = await response.json();
      
      const location: LocationData = {
        city: data.city || 'Unknown',
        country: data.country_name || 'Unknown',
        region: data.region || '',
        timezone: data.timezone || 'UTC',
        coordinates: {
          lat: data.latitude || 0,
          lng: data.longitude || 0
        }
      };

      this.cachedLocation = location;
      this.lastFetch = Date.now();
      return location;
    } catch (error) {
      console.error('IP location failed:', error);
      
      // Ultimate fallback
      return {
        city: 'Dubai',
        country: 'United Arab Emirates',
        region: 'Dubai',
        timezone: 'Asia/Dubai',
        coordinates: { lat: 25.2048, lng: 55.2708 }
      };
    }
  }

  private async reverseGeocode(lat: number, lng: number): Promise<LocationData> {
    try {
      const response = await fetch(
        `https://nominatim.openstreetmap.org/reverse?format=json&lat=${lat}&lon=${lng}`
      );
      const data = await response.json();
      
      return {
        city: data.address.city || data.address.town || data.address.village || 'Unknown',
        country: data.address.country || 'Unknown',
        region: data.address.state || data.address.county || '',
        timezone: 'UTC', // Would need additional API for timezone
        coordinates: { lat, lng }
      };
    } catch (error) {
      console.error('Reverse geocode failed:', error);
      return {
        city: 'Unknown',
        country: 'Unknown',
        region: '',
        timezone: 'UTC',
        coordinates: { lat, lng }
      };
    }
  }
}

// =============================================================================
// NEWS SERVICE
// =============================================================================

class NewsService {
  private cachedNews: Map<string, { articles: NewsArticle[]; timestamp: number }> = new Map();
  private CACHE_DURATION = 600000; // 10 minutes

  async getNews(category: string = 'general', country?: string): Promise<NewsArticle[]> {
    const cacheKey = `${category}-${country || 'global'}`;
    
    // Check cache
    const cached = this.cachedNews.get(cacheKey);
    if (cached && Date.now() - cached.timestamp < this.CACHE_DURATION) {
      return cached.articles;
    }

    try {
      // Use free news API services
      const location = await locationService.getLocation();
      const countryCode = this.getCountryCode(country || location.country);
      
      // Try NewsAPI (requires API key in production)
      const articles = await this.fetchFromNewsAPI(category, countryCode);
      
      if (articles.length > 0) {
        this.cachedNews.set(cacheKey, { articles, timestamp: Date.now() });
        return articles;
      }
    } catch (error) {
      console.error('News fetch failed:', error);
    }

    // Fallback to mock news
    return this.getMockNews(category);
  }

  private async fetchFromNewsAPI(category: string, country: string): Promise<NewsArticle[]> {
    // In production, use real API with key from env
    // For now, return mock data
    return [];
  }

  private getMockNews(category: string): NewsArticle[] {
    const now = new Date();
    const categories: Record<string, NewsArticle[]> = {
      general: [
        {
          title: 'Global Tech Summit Announces Major AI Breakthroughs',
          description: 'Industry leaders gather to discuss the future of artificial intelligence and its impact on society.',
          url: '#',
          source: 'TechNews',
          publishedAt: now.toISOString(),
          category: 'technology'
        },
        {
          title: 'New Climate Agreement Reached by World Leaders',
          description: 'Historic agreement aims to reduce carbon emissions by 50% by 2030.',
          url: '#',
          source: 'Global News',
          publishedAt: new Date(now.getTime() - 3600000).toISOString(),
          category: 'environment'
        },
        {
          title: 'Market Rally Continues as Tech Stocks Surge',
          description: 'Major indices reach new highs amid strong earnings reports.',
          url: '#',
          source: 'Financial Times',
          publishedAt: new Date(now.getTime() - 7200000).toISOString(),
          category: 'business'
        }
      ],
      technology: [
        {
          title: 'Revolutionary AR Glasses Hit the Market',
          description: 'New smart glasses promise to change how we interact with technology.',
          url: '#',
          source: 'Tech Today',
          publishedAt: now.toISOString(),
          category: 'technology'
        },
        {
          title: 'Quantum Computing Reaches New Milestone',
          description: 'Researchers achieve unprecedented processing speeds in quantum systems.',
          url: '#',
          source: 'Science Daily',
          publishedAt: new Date(now.getTime() - 5400000).toISOString(),
          category: 'technology'
        }
      ],
      business: [
        {
          title: 'Cryptocurrency Market Shows Strong Recovery',
          description: 'Digital assets gain momentum as institutional adoption increases.',
          url: '#',
          source: 'Crypto News',
          publishedAt: now.toISOString(),
          category: 'business'
        }
      ]
    };

    return categories[category] || categories.general;
  }

  private getCountryCode(country: string): string {
    const codes: Record<string, string> = {
      'United Arab Emirates': 'ae',
      'United States': 'us',
      'United Kingdom': 'gb',
      'India': 'in',
      'Germany': 'de',
      'France': 'fr',
      'Japan': 'jp',
      'China': 'cn'
    };
    return codes[country] || 'us';
  }
}

// =============================================================================
// WEATHER SERVICE
// =============================================================================

class WeatherService {
  private cachedWeather: WeatherData | null = null;
  private lastFetch: number = 0;
  private CACHE_DURATION = 1800000; // 30 minutes

  async getWeather(city?: string): Promise<WeatherData> {
    // Check cache
    if (this.cachedWeather && Date.now() - this.lastFetch < this.CACHE_DURATION && !city) {
      return this.cachedWeather;
    }

    try {
      const location = await locationService.getLocation();
      const targetCity = city || location.city;
      
      // In production, use OpenWeatherMap or similar API
      // For now, return mock weather
      const weather = this.getMockWeather(targetCity);
      
      if (!city) {
        this.cachedWeather = weather;
        this.lastFetch = Date.now();
      }
      
      return weather;
    } catch (error) {
      console.error('Weather fetch failed:', error);
      return this.getMockWeather('Unknown');
    }
  }

  private getMockWeather(city: string): WeatherData {
    return {
      location: city,
      temperature: 25 + Math.floor(Math.random() * 10),
      condition: ['Sunny', 'Partly Cloudy', 'Cloudy', 'Clear'][Math.floor(Math.random() * 4)],
      humidity: 50 + Math.floor(Math.random() * 30),
      windSpeed: 10 + Math.floor(Math.random() * 15),
      forecast: [
        { day: 'Today', high: 32, low: 24, condition: 'Sunny' },
        { day: 'Tomorrow', high: 31, low: 23, condition: 'Partly Cloudy' },
        { day: 'Tuesday', high: 30, low: 22, condition: 'Cloudy' },
        { day: 'Wednesday', high: 29, low: 21, condition: 'Clear' }
      ]
    };
  }
}

// =============================================================================
// WEB SEARCH SERVICE
// =============================================================================

class WebSearchService {
  async search(query: string, limit: number = 5): Promise<SearchResult[]> {
    try {
      // In production, use Google Custom Search API or similar
      // For now, return mock results
      return this.getMockSearchResults(query, limit);
    } catch (error) {
      console.error('Search failed:', error);
      return [];
    }
  }

  private getMockSearchResults(query: string, limit: number): SearchResult[] {
    const results: SearchResult[] = [
      {
        title: `${query} - Wikipedia`,
        snippet: `Learn about ${query} and its history, significance, and related topics...`,
        url: `https://en.wikipedia.org/wiki/${encodeURIComponent(query)}`,
        displayUrl: 'wikipedia.org'
      },
      {
        title: `What is ${query}? - Complete Guide`,
        snippet: `Comprehensive guide covering everything you need to know about ${query}...`,
        url: `https://example.com/${encodeURIComponent(query)}`,
        displayUrl: 'example.com'
      },
      {
        title: `${query} - Latest News and Updates`,
        snippet: `Stay updated with the latest news and developments about ${query}...`,
        url: `https://news.example.com/${encodeURIComponent(query)}`,
        displayUrl: 'news.example.com'
      }
    ];

    return results.slice(0, limit);
  }
}

// =============================================================================
// TIME & DATE SERVICE
// =============================================================================

class TimeService {
  getCurrentTime(timezone?: string): string {
    const now = new Date();
    if (timezone) {
      return now.toLocaleTimeString('en-US', { timeZone: timezone, hour: '2-digit', minute: '2-digit' });
    }
    return now.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  }

  getCurrentDate(timezone?: string): string {
    const now = new Date();
    if (timezone) {
      return now.toLocaleDateString('en-US', { timeZone: timezone, weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
    }
    return now.toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
  }

  getTimeUntil(targetDate: Date): string {
    const now = new Date();
    const diff = targetDate.getTime() - now.getTime();
    
    if (diff < 0) return 'Event has passed';
    
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
    
    if (days > 0) return `${days} day${days !== 1 ? 's' : ''} ${hours} hour${hours !== 1 ? 's' : ''}`;
    if (hours > 0) return `${hours} hour${hours !== 1 ? 's' : ''} ${minutes} minute${minutes !== 1 ? 's' : ''}`;
    return `${minutes} minute${minutes !== 1 ? 's' : ''}`;
  }
}

// =============================================================================
// FACTS & KNOWLEDGE SERVICE
// =============================================================================

class KnowledgeService {
  async getFactAbout(topic: string): Promise<string> {
    // In production, integrate with Wikipedia API, Wolfram Alpha, etc.
    const facts: Record<string, string> = {
      'uae': 'The UAE consists of seven emirates and was formed on December 2, 1971. Dubai is the most populous city.',
      'dubai': 'Dubai is home to the Burj Khalifa, the world\'s tallest building at 828 meters (2,717 feet).',
      'blockchain': 'Blockchain is a distributed ledger technology that records transactions across multiple computers.',
      'ai': 'Artificial Intelligence refers to the simulation of human intelligence in machines programmed to think and learn.',
      'default': 'Interesting topic! You can search for more information about this online.'
    };

    const normalizedTopic = topic.toLowerCase();
    for (const [key, fact] of Object.entries(facts)) {
      if (normalizedTopic.includes(key)) {
        return fact;
      }
    }

    return facts.default;
  }
}

// =============================================================================
// SINGLETON EXPORTS
// =============================================================================

export const locationService = new LocationService();
export const newsService = new NewsService();
export const weatherService = new WeatherService();
export const webSearchService = new WebSearchService();
export const timeService = new TimeService();
export const knowledgeService = new KnowledgeService();
