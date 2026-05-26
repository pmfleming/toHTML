from __future__ import annotations

from pathlib import Path
import unittest

import pdf_web_compare_app as app


class TextExtractionTests(unittest.TestCase):
    def test_html_parser_keeps_visible_text_and_skips_scripts(self) -> None:
        parser = app.VisibleTextParser()
        parser.feed(
            """
            <html>
              <head><style>.x { color: red }</style></head>
              <body>
                <h1>Title</h1>
                <script>alert("skip")</script>
                <p>Hello <strong>world</strong>.</p>
              </body>
            </html>
            """
        )

        text = parser.text()

        self.assertIn("Title", text)
        self.assertIn("Hello world.", text)
        self.assertNotIn("alert", text)
        self.assertNotIn("color", text)


class ComparisonTests(unittest.TestCase):
    def test_line_stats_counts_source_specific_lines(self) -> None:
        stats = app.line_stats(
            ["alpha", "old line", "shared"],
            ["alpha", "new line", "shared", "web only"],
        )

        self.assertEqual(stats["matching_lines"], 2)
        self.assertEqual(stats["web_only_lines"], 1)
        self.assertEqual(stats["changed_line_blocks"], 1)

    def test_safe_report_name_uses_pdf_and_web_names(self) -> None:
        name = app.safe_report_name(Path("Policy Draft.pdf"), "https://example.com/page")

        self.assertEqual(name, "Policy-Draft-vs-example.com.html")


if __name__ == "__main__":
    unittest.main()
