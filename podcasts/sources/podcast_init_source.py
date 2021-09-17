from datetime import datetime
from typing import List, Optional

from bs4 import BeautifulSoup

from podcasts.sources.base import AbstractPodcastSource, PodcastItem, BaseSource


class PodcastInitSource(BaseSource):
    """https://www.pythonpodcast.com/episodes/"""

    SOURCE_NAME = 'Podcast.__init__'

    def __init__(self, last_processed_item: Optional[PodcastItem] = None) -> None:
        self._last_processed_item = last_processed_item
        self._posts_page_url = 'https://www.pythonpodcast.com/episodes/'

    def set_last_processed(self, item: PodcastItem):
        self._last_processed_item = item

    def get_last_processed(self) -> Optional[PodcastItem]:
        return self._last_processed_item

    def _find_posts_links(self) -> List[str]:
        # TODO: add req and parse
        # return ['https://www.pythonpodcast.com/preql-relational-algebra-sql-replacement-episode-325/']
        raise NotImplementedError()

    def _parse_title(self, bs: BeautifulSoup, class_name: str = 'blog-page-title') -> Optional[str]:
        titles = bs.find_all(**{'class': class_name})
        if titles:
            return titles[0].get_text()

    def _parse_published_at(self, bs: BeautifulSoup) -> Optional[datetime]:
        pass

    def _parse_description(self, bs: BeautifulSoup) -> Optional[str]:
        pass
        
    def get_items_after(self, items_count: int, prev_item: Optional[PodcastItem] = None) -> List[PodcastItem]:
        """Get `items_count` items starting from prev_item(excluded).

        If prev_item is None - return just last `items_count` items

        Args:
            items_count (int): [description]
            prev_item (Optional[PodcastItem]): [description]

        Returns:
            List[PodcastItem]: [description]
        """
        items = []
        # TODO: add filtering based on prev_item and items_count number
        for page_url in self._find_posts_links():
            content = self.get_page_content(url=page_url)
            bs = self.parse_html(content=content)

            item = PodcastItem(
                source=self.SOURCE_NAME,
                title=self._parse_title(bs=bs) or '<no title>',
                url=page_url,
                published_at=self._parse_published_at(bs=bs),
                description=self._parse_description(bs=bs),
            )
            items.append(item)
        return items
