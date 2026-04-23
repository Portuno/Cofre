import axios from 'axios';
import { config } from '../config';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';

export class AudioService {
  private elevenlabsClient = axios.create({
    baseURL: 'https://api.elevenlabs.io/v1',
    headers: {
      'xi-api-key': config.elevenlabs.apiKey,
    },
  });

  async transcribeAudio(audioUrl: string, retries: number = 3): Promise<string> {
    for (let attempt = 0; attempt < retries; attempt++) {
      try {
        // Download audio file
        const audioResponse = await axios.get(audioUrl, {
          responseType: 'arraybuffer',
        });

        const audioBuffer = Buffer.from(audioResponse.data);

        // Send to ElevenLabs for transcription
        const formData = new FormData();
        formData.append('audio', new Blob([audioBuffer], { type: 'audio/mpeg' }));

        const response = await this.elevenlabsClient.post('/speech-to-text', formData, {
          headers: {
            'Content-Type': 'multipart/form-data',
          },
        });

        if (response.data.text) {
          return response.data.text;
        }

        throw new Error('No transcript in response');
      } catch (error) {
        logger.warn({ error, attempt, audioUrl }, `Transcription attempt ${attempt + 1} failed`);

        if (attempt === retries - 1) {
          logger.error({ error, audioUrl }, 'Transcription failed after retries');
          throw new AppError(500, 'TRANSCRIPTION_ERROR', 'Failed to transcribe audio');
        }

        // Exponential backoff
        await new Promise((resolve) => setTimeout(resolve, Math.pow(2, attempt) * 1000));
      }
    }

    throw new AppError(500, 'TRANSCRIPTION_ERROR', 'Failed to transcribe audio');
  }

  async validateAudioFile(buffer: Buffer): Promise<boolean> {
    // Basic validation: check for audio file signatures
    const mp3Signature = Buffer.from([0xff, 0xfb]);
    const wavSignature = Buffer.from([0x52, 0x49, 0x46, 0x46]); // RIFF

    return (
      buffer.slice(0, 2).equals(mp3Signature) ||
      buffer.slice(0, 4).equals(wavSignature)
    );
  }
}

export const audioService = new AudioService();
