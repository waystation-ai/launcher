import Link from 'next/link';

export default function Footer() {
  return (
    <footer className="bg-white/80 rounded-tl-2xl rounded-tr-2xl shadow-[0px_-2px_16px_0px_rgba(0,0,0,0.08)] border-t border-white backdrop-blur-xl px-4 sm:px-6 py-4">
      <div className="max-w-7xl mx-auto flex flex-col sm:flex-row justify-between items-center gap-4">
        <p className="text-sm text-gray-600">© 2025 43D Corporation. All rights reserved.</p>
        <nav className="flex gap-6">
          <Link href="https://waystation.ai/legal/terms" target='_blank' className="text-sm text-gray-600 hover:text-gray-900">Terms of Use</Link>
          <Link href="https://waystation.ai/legal/privacy-policy" target='_blank' className="text-sm text-gray-600 hover:text-gray-900">Privacy Policy</Link>
        </nav>
      </div>
    </footer>
  );
}
