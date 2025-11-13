/**
 * Enhanced localStorage wrapper with error handling and validation
 * Provides robust persistence for TalkToMe settings on Windows and other platforms
 */

export interface PersistenceOptions {
  key: string;
  validate?: (data: any) => boolean;
  migrate?: (data: any) => any;
  fallback?: any;
}

export class LocalStoragePersistence {
  private key: string;
  private validate?: (data: any) => boolean;
  private migrate?: (data: any) => any;
  private fallback?: any;

  constructor(options: PersistenceOptions) {
    this.key = options.key;
    this.validate = options.validate;
    this.migrate = options.migrate;
    this.fallback = options.fallback;
  }

  /**
   * Safely load data from localStorage with error handling
   */
  load(): any {
    try {
      const stored = localStorage.getItem(this.key);
      if (!stored) {
        console.log(`No data found in localStorage for key: ${this.key}`);
        return this.fallback || null;
      }

      const parsed = JSON.parse(stored);

      // Apply migration if available
      let data = parsed;
      if (this.migrate) {
        data = this.migrate(parsed);
      }

      // Validate data if validator provided
      if (this.validate && !this.validate(data)) {
        console.warn(`Validation failed for localStorage key: ${this.key}, using fallback`);
        return this.fallback || null;
      }

      console.log(`Successfully loaded data from localStorage for key: ${this.key}`);
      return data;
    } catch (error) {
      console.error(`Error loading from localStorage for key ${this.key}:`, error);
      // Clear corrupted data
      this.clear();
      return this.fallback || null;
    }
  }

  /**
   * Safely save data to localStorage with error handling
   */
  save(data: any): boolean {
    try {
      // Validate before saving if validator provided
      if (this.validate && !this.validate(data)) {
        console.error(`Validation failed, not saving invalid data for key: ${this.key}`);
        return false;
      }

      const serialized = JSON.stringify(data);
      localStorage.setItem(this.key, serialized);
      console.log(`Successfully saved data to localStorage for key: ${this.key}`);
      return true;
    } catch (error) {
      console.error(`Error saving to localStorage for key ${this.key}:`, error);
      return false;
    }
  }

  /**
   * Clear data from localStorage
   */
  clear(): void {
    try {
      localStorage.removeItem(this.key);
      console.log(`Cleared localStorage data for key: ${this.key}`);
    } catch (error) {
      console.error(`Error clearing localStorage for key ${this.key}:`, error);
    }
  }

  /**
   * Check if data exists and is valid
   */
  exists(): boolean {
    try {
      const data = this.load();
      return data !== null && data !== undefined;
    } catch {
      return false;
    }
  }
}

/**
 * Settings-specific persistence with validation
 */
export class SettingsPersistence extends LocalStoragePersistence {
  constructor() {
    super({
      key: "talktome-settings",
      validate: (data: any) => {
        // Basic validation for settings object
        return (
          data &&
          typeof data === 'object' &&
          typeof data.spokenLanguage === 'string' &&
          typeof data.translationLanguage === 'string' &&
          // Add more validation as needed
          true
        );
      },
      migrate: (data: any) => {
        // Handle legacy data migrations
        let migrated = { ...data };

        // Ensure hotkeys structure exists
        if (!migrated.hotkeys) {
          migrated.hotkeys = { handsFree: "Ctrl+Shift+Space" };
        }

        // Force audio chunking to false for reliability
        migrated.audioChunkingEnabled = false;

        // Remove API key if present (security)
        if (migrated.apiKey) {
          delete migrated.apiKey;
        }

        return migrated;
      }
    });
  }
}

// Export singleton instance for settings
export const settingsPersistence = new SettingsPersistence();
