import React from 'react';
import { Image as ImageIcon, Heart, Share2 } from 'lucide-react';

const MOCK_PHOTOS = [
    'https://picsum.photos/400/600?random=1',
    'https://picsum.photos/400/600?random=2',
    'https://picsum.photos/400/600?random=3',
    'https://picsum.photos/400/600?random=4',
    'https://picsum.photos/400/600?random=5',
    'https://picsum.photos/400/600?random=6',
];

export const GalleryApp: React.FC = () => {
  return (
    <div className="flex flex-col h-full bg-black text-white rounded-b-3xl overflow-hidden">
      <div className="p-4 grid grid-cols-2 gap-4 overflow-y-auto">
        {MOCK_PHOTOS.map((url, i) => (
            <div key={i} className="relative aspect-[3/4] group rounded-xl overflow-hidden bg-zinc-900 border border-white/10">
                <img src={url} alt={`Gallery ${i}`} className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110" />
                <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity flex items-end justify-between p-3">
                    <button className="p-2 hover:bg-white/20 rounded-full transition-colors"><Heart size={16} /></button>
                    <button className="p-2 hover:bg-white/20 rounded-full transition-colors"><Share2 size={16} /></button>
                </div>
            </div>
        ))}
      </div>
    </div>
  );
};
