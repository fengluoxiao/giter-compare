import { invoke } from '@tauri-apps/api/core';

export interface GrammarPlugin {
  id: string;
  name: string;
  version: string;
  description: string;
  languages: string[];
  grammarPath: string;
  enabled: boolean;
  conflictsWith?: string[];
}

export interface GrammarDefinition {
  scopeName: string;
  fileTypes: string[];
  injectionSelector?: string;
  patterns: any[];
  repository?: Record<string, any>;
}

const PLUGIN_DIR = '.gitercompare/plugins';
const PLUGIN_CONFIG_FILE = '.gitercompare/plugins.json';

class PluginManager {
  private plugins: Map<string, GrammarPlugin> = new Map();
  private grammars: Map<string, GrammarDefinition> = new Map();
  private loaded: boolean = false;

  async init(): Promise<void> {
    if (this.loaded) return;
    await this.loadPlugins();
    this.loaded = true;
  }

  async loadPlugins(): Promise<void> {
    try {
      const config = await this.loadPluginConfig();
      if (config && config.plugins) {
        for (const plugin of config.plugins) {
          this.plugins.set(plugin.id, plugin);
          if (plugin.enabled) {
            await this.loadGrammar(plugin);
          }
        }
      }
    } catch (e) {
      console.error('Failed to load plugins:', e);
    }
  }

  async loadPluginConfig(): Promise<{ plugins: GrammarPlugin[] } | null> {
    try {
      const homeDir = await invoke<string>('get_home_dir');
      const configPath = `${homeDir}/${PLUGIN_CONFIG_FILE}`;
      const content = await invoke<string>('read_file', { path: configPath });
      return JSON.parse(content);
    } catch (e) {
      return null;
    }
  }

  async savePluginConfig(): Promise<void> {
    try {
      const homeDir = await invoke<string>('get_home_dir');
      const configPath = `${homeDir}/${PLUGIN_CONFIG_FILE}`;
      const config = {
        plugins: Array.from(this.plugins.values())
      };
      await invoke('write_file', {
        path: configPath,
        content: JSON.stringify(config, null, 2)
      });
    } catch (e) {
      console.error('Failed to save plugin config:', e);
    }
  }

  async loadGrammar(plugin: GrammarPlugin): Promise<void> {
    try {
      const content = await invoke<string>('read_file', { path: plugin.grammarPath });
      const grammar: GrammarDefinition = JSON.parse(content);
      
      for (const lang of plugin.languages) {
        this.grammars.set(lang, grammar);
      }
    } catch (e) {
      console.error(`Failed to load grammar for plugin ${plugin.id}:`, e);
    }
  }

  async installPlugin(pluginPath: string): Promise<{ success: boolean; message: string; conflicts?: string[] }> {
    try {
      // 读取插件信息
      const packageJsonPath = `${pluginPath}/package.json`;
      const packageContent = await invoke<string>('read_file', { path: packageJsonPath });
      const packageJson = JSON.parse(packageContent);

      const plugin: GrammarPlugin = {
        id: packageJson.name,
        name: packageJson.displayName || packageJson.name,
        version: packageJson.version,
        description: packageJson.description || '',
        languages: packageJson.contributes?.languages?.map((l: any) => l.id) || [],
        grammarPath: `${pluginPath}/${packageJson.contributes?.grammars?.[0]?.path || 'syntaxes/grammar.tmLanguage.json'}`,
        enabled: true
      };

      // 检查冲突
      const conflicts = this.checkConflicts(plugin);
      if (conflicts.length > 0) {
        return {
          success: false,
          message: `Plugin conflicts with: ${conflicts.join(', ')}`,
          conflicts
        };
      }

      // 复制插件到用户目录
      const homeDir = await invoke<string>('get_home_dir');
      const targetDir = `${homeDir}/${PLUGIN_DIR}/${plugin.id}`;
      await invoke('copy_dir', { from: pluginPath, to: targetDir });

      // 更新路径
      plugin.grammarPath = `${targetDir}/${packageJson.contributes?.grammars?.[0]?.path || 'syntaxes/grammar.tmLanguage.json'}`;

      // 添加到插件列表
      this.plugins.set(plugin.id, plugin);
      await this.loadGrammar(plugin);
      await this.savePluginConfig();

      return { success: true, message: `Plugin ${plugin.name} installed successfully` };
    } catch (e) {
      return { success: false, message: `Failed to install plugin: ${e}` };
    }
  }

  checkConflicts(plugin: GrammarPlugin): string[] {
    const conflicts: string[] = [];
    
    for (const [, existingPlugin] of this.plugins) {
      if (!existingPlugin.enabled) continue;
      
      // 检查语言冲突
      for (const lang of plugin.languages) {
        if (existingPlugin.languages.includes(lang)) {
          conflicts.push(`${existingPlugin.name} (${lang})`);
        }
      }
    }

    return conflicts;
  }

  async enablePlugin(pluginId: string): Promise<boolean> {
    const plugin = this.plugins.get(pluginId);
    if (!plugin) return false;

    plugin.enabled = true;
    await this.loadGrammar(plugin);
    await this.savePluginConfig();
    return true;
  }

  async disablePlugin(pluginId: string): Promise<boolean> {
    const plugin = this.plugins.get(pluginId);
    if (!plugin) return false;

    plugin.enabled = false;
    
    // 移除语法定义
    for (const lang of plugin.languages) {
      this.grammars.delete(lang);
    }
    
    await this.savePluginConfig();
    return true;
  }

  async uninstallPlugin(pluginId: string): Promise<boolean> {
    const plugin = this.plugins.get(pluginId);
    if (!plugin) return false;

    // 移除语法定义
    for (const lang of plugin.languages) {
      this.grammars.delete(lang);
    }

    // 删除插件目录
    try {
      const homeDir = await invoke<string>('get_home_dir');
      const pluginDir = `${homeDir}/${PLUGIN_DIR}/${pluginId}`;
      await invoke('remove_dir', { path: pluginDir });
    } catch (e) {
      console.error('Failed to remove plugin directory:', e);
    }

    this.plugins.delete(pluginId);
    await this.savePluginConfig();
    return true;
  }

  getPlugins(): GrammarPlugin[] {
    return Array.from(this.plugins.values());
  }

  getGrammar(language: string): GrammarDefinition | undefined {
    return this.grammars.get(language);
  }

  getGrammarByScopeName(scopeName: string): GrammarDefinition | undefined {
    for (const grammar of this.grammars.values()) {
      if (grammar.scopeName === scopeName) {
        return grammar;
      }
    }
    return undefined;
  }

  hasGrammar(language: string): boolean {
    return this.grammars.has(language);
  }
}

export const pluginManager = new PluginManager();
