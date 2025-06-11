import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useState } from 'react';

export default function Compressor() {
  const [sourceDir, setSourceDir] = useState('');
  const [outputFile, setOutputFile] = useState('');
  const [status, setStatus] = useState('');

  const selectFolder = async () => {
    const selected = await open({
      directory: true,
    });
    if (selected) {
      setSourceDir(selected as string);
    }
  };

  const selectOutput = async () => {
    const selected = await open({
      title: '选择输出目录',
      directory: true  // 改为选择目录而不是文件
    });
    if (selected) {
      // 自动生成ZIP文件名，例如使用源文件夹名 + .zip
      const folderName = sourceDir.split(/[\\/]/).pop() || 'archive';
      setOutputFile(`${selected}/${folderName}.zip`);
    }
  };

  const compress = async () => {
    if (!sourceDir || !outputFile) {
      setStatus('请先选择文件夹和输出文件');
      return;
    }

    setStatus('压缩中...');
    try {
      const result = await invoke('compress_folder', {
        sourceDir,
        outputFile
      });
      setStatus(result as string);

      // 尝试打开压缩后的文件
      try {
        await invoke('open_file', { path: outputFile });
      } catch (openError) {
        console.error('打开文件失败:', openError);
      }
    } catch (error) {
      setStatus(`压缩失败: ${error}`);
      console.error('详细错误:', error);
    }
};

  return (
    <div>
      <button onClick={selectFolder}>选择要压缩的文件夹</button>
      <div>源文件夹: {sourceDir}</div>

      <button onClick={selectOutput}>选择输出位置</button>
      <div>输出文件: {outputFile}</div>

      <button onClick={compress}>开始压缩</button>
      <div>状态: {status}</div>
    </div>
  );
}
