from abc import ABC, abstractmethod

from datetime import datetime
from typing import Callable, List, Optional
import pydantic
from uuid import UUID, uuid4


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

class AbstractPodcastSource:
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


# podcast observer
class PodcastsNotificationService:
    def __init__(self) -> None:
        self._sources: List[AbstractPodcastSource] = []
        self._subscibers: List[Callable] = []

    def register_source(self, source: AbstractPodcastSource):
        if not isinstance(source, AbstractPodcastSource):
            raise ValueError(f'Invalid type of source: {source}. Should instance of AbstractPodcastSource')
        self._sources.append(source)

    def register_subsciber(self, subsciber: Callable):
        if not callable(subsciber):
            raise ValueError(f'subsciber should be a callable object')
        self._subscibers.append(subsciber)

    def notify_about_new_item(self, item: PodcastItem):
        for sub in self._subscibers:
            sub(item)

    def check_new_podcasts(self):
        # TODO: do that using parallel processing
        for source in self._sources:
            item = source.get_last_processed()
            # TODO: add explicit sorting
            items = source.get_items_after(item) or []
            for item in items:
                self.notify_about_new_item(item)

            source.set_last_processed(item)
