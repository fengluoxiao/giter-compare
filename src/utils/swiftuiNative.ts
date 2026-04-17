import { invoke } from '@tauri-apps/api/core';

export interface OpenPanelOptions {
  content: string;
  title?: string;
  button_text?: string;
}

export interface ShowAlertOptions {
  title: string;
  message: string;
  button_text?: string;
}

export interface SwiftUIResponse {
  status: string;
  message?: string;
  action?: string;
}

/**
 * 打开 SwiftUI 原生面板
 */
export async function openSwiftUIPanel(options: OpenPanelOptions): Promise<SwiftUIResponse> {
  try {
    const response = await invoke<SwiftUIResponse>('open_swiftui_panel', {
      request: options
    });
    console.log('SwiftUI 面板响应:', response);
    return response;
  } catch (error) {
    console.error('打开 SwiftUI 面板失败:', error);
    throw error;
  }
}

/**
 * 显示原生警告弹窗
 */
export async function showNativeAlert(options: ShowAlertOptions): Promise<SwiftUIResponse> {
  try {
    const response = await invoke<SwiftUIResponse>('show_native_alert', {
      request: options
    });
    console.log('原生警告响应:', response);
    return response;
  } catch (error) {
    console.error('显示原生警告失败:', error);
    throw error;
  }
}

/**
 * 快速显示信息弹窗
 */
export async function showInfo(message: string, title: string = '提示'): Promise<void> {
  await showNativeAlert({
    title,
    message,
    button_text: '确定'
  });
}

/**
 * 快速显示确认弹窗
 */
export async function showConfirm(message: string, title: string = '确认'): Promise<boolean> {
  try {
    const result = await showNativeAlert({
      title,
      message,
      button_text: '确认'
    });
    return result.status === 'confirmed';
  } catch {
    return false;
  }
}
