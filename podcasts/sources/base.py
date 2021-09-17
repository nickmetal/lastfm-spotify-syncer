from abc import ABC, abstractmethod
from uuid import UUID, uuid4
from datetime import datetime
from typing import Dict, List, Optional

import pydantic
import requests
from bs4 import BeautifulSoup


class PodcastItem(pydantic.BaseModel):
    item_id: UUID = pydantic.Field(default_factory=uuid4)
    url: str  # direct media url or url to the post that contains audio/video data
    title: str
    source: str  # source name
    description: Optional[str]
    published_at: Optional[datetime]

    def __eq__(self, other: "PodcastItem") -> bool:
        return (
            self.url,
            self.published_at
        ) == (
            other.url,
            other.published_at
        )

class AbstractPodcastSource(ABC):
    @abstractmethod
    def set_last_processed(item: PodcastItem):
        pass

    @abstractmethod
    def get_last_processed() -> Optional[PodcastItem]:
        pass

    @abstractmethod
    def get_items_after(items_count: int, prev_item: Optional[PodcastItem]) -> List[PodcastItem]:
        """Get `items_count` items starting from prev_item(excluded).

        If prev_item is None - return just last `items_count` items

        Args:
            items_count (int): [description]
            prev_item (Optional[PodcastItem]): [description]

        Returns:
            List[PodcastItem]: [description]
        """


class BaseSource(AbstractPodcastSource):
    def get_page_content(self, url: str, headers: Optional[Dict] = None) -> Optional[str]:
        response = requests.get(url, headers=headers)
        if response.ok:
            return response.content
        else:
            response.raise_for_status()

    def parse_html(self, content: str, parser: Optional[str] = 'html.parser') -> BeautifulSoup:
        return BeautifulSoup(content, parser)